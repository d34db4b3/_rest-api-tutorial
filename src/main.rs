use actix_web::{get, web, App, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GreetQuery {
    name: String,
}

#[derive(Serialize)]
struct GreetResponse {
    greeting: String,
}

#[get("/greet")]
async fn greet(greet_query: web::Query<GreetQuery>) -> web::Json<GreetResponse> {
    web::Json(GreetResponse {
        greeting: format!("Hello, {}!", greet_query.name),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
