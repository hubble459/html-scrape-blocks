use std::{
    any::{type_name, Any},
    fmt::Debug, sync::Arc,
};

use kuchiki::traits::{ElementIterator, NodeIterator};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

use super::{query_matcher::QueryMatcher, scrape_block_error::ScrapeBlockError};

fn downcast<T: 'static>(value: Result<Box<dyn Any>, ScrapeBlockError>) -> Result<T, ScrapeBlockError> {
    match value {
        Ok(value) => value.downcast::<T>().map_or(
            Err(ScrapeBlockError::InvalidType {
                expected: String::from(type_name::<T>()),
                found: "Unknown".to_owned(),
            }),
            |v| Ok(*v),
        ),
        Err(e) => Err(e),
    }
}

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum Matcher {
    Array {
        query: QueryMatcher,
        each: Box<Matcher>,
    },
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
    StringArray {
        query: QueryMatcher,
        split_regex: Option<String>,
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
            Matcher::Array { query, .. } => query,
            Matcher::String { query } => query,
            Matcher::Integer { query } => query,
            Matcher::Double { query } => query,
            Matcher::URL { query } => query,
            Matcher::Date { query, .. } => query,
            Matcher::Boolean { query, .. } => query,
            Matcher::StringArray { query, .. } => query,
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

        let elements = query.select(node.clone());
        debug!("\tFound {} elements", elements.clone().count());
        return match self {
            Matcher::Array { each, .. } => {
                let mut collect = vec![];
                for element in elements {
                    info!("el: {}", element.name.local.as_ref());
                    collect.push(each.exec(element.as_node().inclusive_descendants().elements()));
                }
                Ok(Box::new(collect))
            }
            Matcher::String { query } => {
                let text = query.text(elements)?;
                Ok(Box::new(text))
            }
            Matcher::Integer { query } => {
                let text = query.text(elements)?;
                Ok(Box::new(
                    i64::from_str_radix(&text, 10)
                        .map_err(|_e| ScrapeBlockError::NotAnInteger(text)),
                ))
            }
            Matcher::Double { query } => {
                let text = query.text(elements)?;
                Ok(Box::new(
                    text.parse::<f64>()
                        .map_err(|_e| ScrapeBlockError::NotAnInteger(text)),
                ))
            }
            Matcher::Boolean {
                query,
                matches_regex,
            } => {
                let text = query.text(elements)?;
                let matches = RegexBuilder::new(&matches_regex)
                    .case_insensitive(true)
                    .build()
                    .map_err(|e| ScrapeBlockError::InvalidRegex(e))?
                    .is_match(&text);
                debug!("\t`{text}` matches regex {matches_regex}: {matches}");
                Ok(Box::new(matches))
            }
            Matcher::URL { query } => {
                let text = query.text(elements)?;
                Ok(Box::new(
                    reqwest::Url::parse(&text)
                        .map_err(|_| ScrapeBlockError::UrlParseError(text))?,
                ))
            }
            Matcher::Date {
                query,
                date_formats,
            } => {
                let text = query.text(elements)?;
                for date_format in date_formats {
                    #[cfg(feature = "chrono")]
                    {
                        let date = chrono::NaiveDateTime::parse_from_str(&text, &date_format);
                        match date {
                            Ok(date) => return Ok(Box::new(date)),
                            Err(e) => match e.kind() {
                                chrono::format::ParseErrorKind::BadFormat => {
                                    return Err(ScrapeBlockError::InvalidDateFormat(
                                        date_format.to_string(),
                                        Some(e),
                                    ))
                                }
                                _ => {}
                            },
                        }
                    }
                    #[cfg(feature = "time")]
                    {
                        let date = time::PrimitiveDateTime::parse(
                            &text,
                            &time::format_description::parse(&date_format).map_err(|_e| {
                                ScrapeBlockError::InvalidDateFormat(date_format.to_string(), None)
                            })?,
                        );
                        if let Ok(date) = date {
                            return Ok(Box::new(date));
                        }
                    }
                }
                Err(ScrapeBlockError::NotADate(text))
            }
            Matcher::StringArray { split_regex, .. } => {
                let mut strings = vec![];
                let split_regex = if let Some(split_regex) = split_regex {
                    Some(Regex::new(&split_regex).map_err(|e| ScrapeBlockError::InvalidRegex(e))?)
                } else {
                    None
                };

                for element in elements {
                    let text = element.text_contents();
                    if let Some(regex) = &split_regex {
                        for part in regex.split(&text) {
                            strings.push(part.to_string());
                        }
                    } else {
                        strings.push(text);
                    }
                }
                Ok(Box::new(strings))
            }
            Matcher::Condition {
                query,
                matches_regex,
                if_true,
                if_false,
            } => {
                let mut matches = elements.clone().count() != 0;

                debug!("\texists: {matches}");
                if matches {
                    let text = query.text(elements)?;
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

    pub fn exec_downcase<T: ElementIterator + Clone, R: 'static>(
        &self,
        node: T,
    ) -> Result<R, ScrapeBlockError> {
        let value = self.exec(node);
        downcast(value)
    }

    pub fn exec_string<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<String, ScrapeBlockError> {
        self.exec_downcase(node)
    }

    pub fn exec_string_array<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<Vec<String>, ScrapeBlockError> {
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

    #[cfg(feature = "chrono")]
    pub fn exec_date<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<chrono::NaiveDateTime, ScrapeBlockError> {
        self.exec_downcase(node)
    }

    #[cfg(feature = "time")]
    pub fn exec_date<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<time::PrimitiveDateTime, ScrapeBlockError> {
        self.exec_downcase(node)
    }

    pub fn exec_url<T: ElementIterator + Clone>(
        &self,
        node: T,
    ) -> Result<reqwest::Url, ScrapeBlockError> {
        self.exec_downcase(node)
    }

    pub fn exec_array<T: ElementIterator + Clone, ArrayType: 'static>(
        &self,
        node: T,
    ) -> Result<Vec<ArrayType>, ScrapeBlockError> {
        let items = self.exec_downcase::<_, Vec<Result<Box<dyn Any>, ScrapeBlockError>>>(node)?;
        items.into_iter().map(|item| downcast(item)).collect()
    }
}
