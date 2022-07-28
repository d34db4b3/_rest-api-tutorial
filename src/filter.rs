use crate::{errors::ContentTypeError, utils::regex::RegexWrapper};
use actix_web::{
    body::BoxBody,
    http::header::{Accept, ContentType, Header},
    post,
    web::{self},
    FromRequest, HttpResponse, Responder, ResponseError,
};
use serde::{Deserialize, Serialize};
use split_iter::Splittable;
use std::{future::Future, pin::Pin};

#[derive(Deserialize)]
struct Filter {
    include: RegexWrapper,
    exclude: Option<RegexWrapper>,
}

#[derive(Deserialize)]
pub struct FilterRequest {
    lines: Vec<String>,
    #[serde(flatten)]
    filter: Filter,
}

impl FromRequest for FilterRequest {
    type Error = ContentTypeError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match ContentType::parse(req) {
            Ok(accept) => match accept.essence_str() {
                "application/json" => {
                    let json_fut = web::Json::<Self>::from_request(req, payload);
                    return Box::pin(async {
                        match json_fut.await {
                            Ok(v) => Ok(v.into_inner()),
                            Err(e) => Err(ContentTypeError::Other(e)),
                        }
                    });
                }
                "text/plain" => {
                    let query_fut = web::Query::<Filter>::from_request(req, payload);
                    let plain_fut = String::from_request(req, payload);
                    return Box::pin(async {
                        match plain_fut.await {
                            Ok(s) => match query_fut.await {
                                Ok(query) => Ok(Self {
                                    lines: s.lines().map(str::to_string).collect(),
                                    filter: query.0,
                                }),
                                Err(e) => Err(ContentTypeError::Other(e)),
                            },
                            Err(e) => Err(ContentTypeError::Other(e)),
                        }
                    });
                }
                _ => {}
            },
            _ => {}
        }
        Box::pin(async { Err(ContentTypeError::UnsupportedMediaType) })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterResponse {
    matching: Vec<String>,
    excluded: Vec<String>,
    not_matching: Vec<String>,
}

impl Responder for FilterResponse {
    type Body = BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match Accept::parse(req) {
            Ok(accept) => {
                for accept in accept.ranked() {
                    match accept.essence_str() {
                        "application/json" => {
                            return web::Json(self).respond_to(req).map_into_boxed_body();
                        }
                        "text/plain" => {
                            return if self.matching.is_empty() {
                                HttpResponse::NoContent().finish()
                            } else {
                                HttpResponse::Ok().body(self.matching.join("\n"))
                            };
                        }
                        _ => {}
                    }
                }
                HttpResponse::NotAcceptable().finish()
            }
            Err(e) => e.error_response(),
        }
    }
}

#[post("/filter")]
pub async fn filter(filter_request: FilterRequest) -> FilterResponse {
    let (rest, excluded) = filter_request.lines.into_iter().split(|line| {
        if let Some(exclude) = &filter_request.filter.exclude {
            return exclude.is_match(line);
        }
        false
    });
    let (not_matching, matching) = rest.split(|line| filter_request.filter.include.is_match(line));
    FilterResponse {
        matching: matching.collect(),
        excluded: excluded.collect(),
        not_matching: not_matching.collect(),
    }
}
