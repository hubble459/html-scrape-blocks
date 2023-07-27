use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScrapeBlockError {
    #[error("This regex is invalid: `{0}`")]
    InvalidRegex(#[from] regex::Error),
    #[error("This url is invalid: `{0}`")]
    UrlParseError(String),
    #[error("Expected to find an element but got None: `{0}`")]
    ElementExpected(String),
    #[error("Selector `{0}` resulted in an empty text")]
    EmptyText(String),
    #[error("ScrapeBlock returned type of {found:?}, but expected {expected:?}")]
    InvalidType { expected: String, found: String },
    #[error("Expected an integer, but got: `{0}`")]
    NotAnInteger(String),
    #[error("Expected a double, but got: `{0}`")]
    NotADouble(String),
    #[error("Expected a date, but got: `{0}`")]
    NotADate(String),
    #[error("Invalid date format: `{0}` {1:#?}")]
    InvalidDateFormat(String, Option<chrono::ParseError>),
}
