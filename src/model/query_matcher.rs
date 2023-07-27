use kuchiki::{iter::Select, traits::ElementIterator};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::util::kuchiki_elements::ElementsTrait;

use super::scrape_block_error::ScrapeBlockError;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum TextType {
    #[default]
    Own,
    All {
        join_str: String,
    },
    Attribute {
        join_str: Option<String>,
        attributes: Vec<String>,
    },
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct QueryMatcher {
    pub selector: String,
    pub text_type: TextType,
    pub clean_with_regex_1: Option<String>,
    pub default: Option<String>,
}

impl QueryMatcher {
    pub fn select<T: ElementIterator + Clone>(&self, node: T) -> Select<T> {
        node.select(self.selector.as_str()).expect("Invalid Regex")
    }

    pub fn text<T: ElementIterator + Clone>(&self, node: T) -> Result<String, ScrapeBlockError> {
        let text = match &self.text_type {
            TextType::Own => node.own_text(),
            TextType::All { join_str } => node.all_text(&join_str),
            TextType::Attribute {
                join_str,
                attributes,
            } => {
                if let Some(join_str) = join_str {
                    node.attrs_first_of(attributes.as_slice()).join(&join_str)
                } else {
                    node.attr_first_of(attributes.as_slice())
                        .unwrap_or_default()
                }
            }
        };

        debug!("\tfound text: {text}");

        if text.is_empty() {
            debug!("\nusing default text: {text}");

            return self.default.clone().map_or(
                Err(ScrapeBlockError::ElementExpected(self.selector.clone())),
                |default| Ok(default),
            );
        } else if let Some(regex) = &self.clean_with_regex_1 {
            let regex = Regex::new(regex).expect("Bad regex `clean_with_regex_1`");
            if let Some(clean_text) = regex.find(&text) {
                let clean_text = clean_text.as_str().to_string();
                debug!("\nclean text: {text}");
                Ok(clean_text)
            } else {
                Err(ScrapeBlockError::ElementExpected(self.selector.clone()))
            }
        } else {
            return Ok(text);
        }
    }
}
