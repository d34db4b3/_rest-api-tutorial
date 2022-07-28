use regex::Regex;
use serde::{de, Deserialize, Deserializer};
use std::ops::Deref;

#[derive(Deserialize)]
pub struct RegexWrapper(#[serde(deserialize_with = "deserialize_regex")] Regex);

impl Deref for RegexWrapper {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn deserialize_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    Regex::new(&s).map_err(de::Error::custom)
}
