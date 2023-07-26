use std::{
    any::{type_name, Any},
    fmt::{Debug, Display},
};

use kuchiki::traits::ElementIterator;
use regex::{Regex, RegexBuilder};
use serde::{de::value, Deserialize, Serialize};

use super::{query_matcher::QueryMatcher, scrape_block_error::ScrapeBlockError};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum Matcher {
    String {
        query: QueryMatcher,
    },
    Integer {
        query: QueryMatcher,
    },
    Double {
        query: QueryMatcher,
    },
    URL {
        query: QueryMatcher,
    },
    Date {
        query: QueryMatcher,
        date_formats: Vec<String>,
    },
    Boolean {
        query: QueryMatcher,
        matches_regex: String,
    },
    Array {
        query: QueryMatcher,
        split: Option<String>,
        regex: Option<String>,
    },
    Condition {
        query: QueryMatcher,
        matches_regex: Option<String>,
        if_true: Box<Matcher>,
        if_false: Box<Matcher>,
    },
}

impl Matcher {
    pub fn query(&self) -> QueryMatcher {
        match self {
            Matcher::String { query } => query,
            Matcher::Integer { query } => query,
            Matcher::Double { query } => query,
            Matcher::URL { query } => query,
            Matcher::Date { query, .. } => query,
            Matcher::Boolean { query, .. } => query,
            Matcher::Array { query, .. } => query,
            Matcher::Condition { query, .. } => query,
        }
        .clone()
    }

    pub fn exec<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<Box<dyn Any>, ScrapeBlockError> {
        let query = self.query();
        debug!("{}: {}", self.as_ref(), query.selector);

        let element = query.select(node.clone());
        debug!("\tFound {} elements", element.clone().count());

        return match self {
            Matcher::String { query } => {
                let text = query.text(element)?;
                debug!("\tfound text: {text}");
                Ok(Box::new(text))
            }
            Matcher::URL { query } => todo!(),
            Matcher::Integer { query } => todo!(),
            Matcher::Double { query } => todo!(),
            Matcher::Boolean {
                query,
                matches_regex,
            } => {
                let text = query.text(element)?;
                let matches = RegexBuilder::new(&matches_regex)
                    .case_insensitive(true)
                    .build()
                    .map_err(|e| ScrapeBlockError::InvalidRegex(e))?
                    .is_match(&text);
                debug!("\t`{text}` matches regex {matches_regex}: {matches}");
                Ok(Box::new(matches))
            }
            Matcher::Date {
                query,
                date_formats,
            } => todo!(),
            Matcher::Array {
                query,
                split,
                regex,
            } => todo!(),
            Matcher::Condition {
                query,
                matches_regex,
                if_true,
                if_false,
            } => {
                let mut matches = element.clone().count() != 0;

                debug!("\texists: {matches}");
                if matches {
                    let text = query.text(element)?;
                    if let Some(matches_regex) = matches_regex {
                        matches = RegexBuilder::new(&matches_regex)
                            .case_insensitive(true)
                            .build()
                            .map_err(|e| ScrapeBlockError::InvalidRegex(e))?
                            .is_match(&text);
                        debug!("\t`{text}` matches regex {matches_regex}: {matches}");
                    }
                }

                debug!("\tgoing into: if_{matches}");
                if matches {
                    return if_true.exec(node);
                } else {
                    return if_false.exec(node);
                }
            }
        };
    }

    pub fn exec_downcase<T: ElementIterator + Clone, R: Clone + 'static>(
        &self,
        node: T,
    ) -> Result<R, ScrapeBlockError> {
        let value = self.exec(node);
        match value {
            Ok(value) => value.downcast_ref::<R>().map_or(
                Err(ScrapeBlockError::InvalidType {
                    expected: String::from(type_name::<R>()),
                    found: "Unknown".to_owned(),
                }),
                |v| Ok(v.clone()),
            ),
            Err(e) => Err(e),
        }
    }

    pub fn exec_string<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<String, ScrapeBlockError> {
        self.exec_downcase(node)
    }

    pub fn exec_boolean<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<bool, ScrapeBlockError> {
        self.exec_downcase(node)
    }

    pub fn exec_integer<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<i64, ScrapeBlockError> {
        self.exec_downcase(node)
    }

    pub fn exec_double<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<f64, ScrapeBlockError> {
        self.exec_downcase(node)
    }

    pub fn exec_date<T: ElementIterator + Clone>(&self, node: T) -> Result<bool, ScrapeBlockError> {
        // todo chrono::NaiveDate
        self.exec_downcase(node)
    }

    pub fn exec_url<T: ElementIterator + Clone>(&self, node: T) -> Result<bool, ScrapeBlockError> {
        // todo reqwest::Url
        self.exec_downcase(node)
    }
}
