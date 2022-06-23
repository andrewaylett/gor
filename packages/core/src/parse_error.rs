use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

/// Something really dodgy is going on
#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum InternalError {
    // Just giving up
    #[error("Error: {0}")]
    Error(String),
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"^(.*?)\((.*)\)$").unwrap();
}

pub fn parse_enum(input: &str) -> Result<(&str, &str), InternalError> {
    let caps = RE.captures(input);
    caps.and_then(|caps| match (caps.get(1), caps.get(2)) {
        (Some(name), Some(param)) => Some((name.as_str(), param.as_str())),
        _ => None,
    })
    .ok_or_else(|| InternalError::Error(format!("Failed to match: {}", input)))
}
