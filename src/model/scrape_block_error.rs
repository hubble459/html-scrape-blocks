use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScrapeBlockError {
    #[error("This regex is invalid: `{0}`")]
    InvalidRegex(#[from] regex::Error),
    #[error("Expected to find an element but got None: `{0}`")]
    ElementExpected(String),
    #[error("Selector `{0}` resulted in an empty text")]
    EmptyText(String),
    #[error("ScrapeBlock returned type of {found:?}, but expected {expected:?}")]
    InvalidType { expected: String, found: String },
}
