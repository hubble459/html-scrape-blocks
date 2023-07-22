use std::{any::Any, collections::HashMap, error::Error};

use kuchiki::{
    iter::{Descendants, Elements, Select},
    traits::{ElementIterator, TendrilSink},
    NodeRef,
};
use serde::{Deserialize, Serialize};
use util::kuchiki_elements::ElementsTrait;
pub mod util;

type Element = Select<Elements<Descendants>>;

fn main() {
    println!("Hello, world!");

    let mut matchers = HashMap::<u32, Matcher>::new();
    let mut testers = HashMap::<u32, Tester>::new();
    let mut scrapers = HashMap::<u32, Scraper>::new();

    /*
    table: matcher
    id: uint32
    matcher: json

    table: tester
    id: uint32
    tester: json

    table: scraper
    id: uint32
    tester_id: uint32
    matcher_id: uint32
    field: string
     */

    let condition = Matcher::Condition {
        query: QueryMatcher {
            selector: "h1.title".to_string(),
            text_type: TextType::Own,
            clean_with_regex_1: None,
        },
        matches_regex: "regex".to_string(),
        if_true: Box::new(Matcher::String {
            query: QueryMatcher {
                selector: "h1.title".to_string(),
                text_type: TextType::Own,
                clean_with_regex_1: None,
            },
        }),
        if_false: None,
    };

    println!("{}", serde_json::json!(condition).to_string());
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Scraper {
    tester_id: u32,
    matcher_id: u32,
    field: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Tester {
    query: QueryMatcher,
    matches_regex: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(tag = "type")]
enum TextType {
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

#[derive(Debug, Default, Deserialize, Serialize)]
struct QueryMatcher {
    selector: String,
    text_type: TextType,
    clean_with_regex_1: Option<String>,
}

impl QueryMatcher {
    fn select<T: ElementIterator>(&self, node: T) -> Select<T> {
        node.select(self.selector.as_str()).expect("Invalid Regex")
    }

    fn text<T: ElementIterator>(&self, node: T) -> String {
        match self.text_type {
            TextType::Own => node.own_text(),
            TextType::All { join_str } => node.all_text(&join_str),
            TextType::Attribute {
                join_str,
                attributes,
            } => {
                if let Some(join_str) = join_str {
                    node.attrs_first_of(attributes.as_slice()).join(&join_str)
                } else {
                    node.attr_first_of(attributes.as_slice()).unwrap_or_default()
                }
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Matcher {
    String {
        query: QueryMatcher,
    },
    Number {
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
        matches_regex: String,
        if_true: Box<Matcher>,
        if_false: Option<Box<Matcher>>,
    },
}

impl Matcher {
    pub fn exec(&self, node: Element) -> Box<dyn Any> {
        match self {
            Matcher::String { query } => {
                let element = query.select(node);
                query.text(element)
            }
            Matcher::Number { query } => todo!(),
            Matcher::URL { query } => todo!(),
            Matcher::Date {
                query,
                date_formats,
            } => todo!(),
            Matcher::Boolean {
                query,
                matches_regex,
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
            } => todo!(),
        }
        todo!()
    }
}

#[cfg(test)]
fn make_cfg() -> Matcher {
    Matcher::Condition {
        query: QueryMatcher {
            selector: "h1.title".to_string(),
            text_type: TextType::Own,
            clean_with_regex_1: None,
        },
        matches_regex: "regex".to_string(),
        if_true: Box::new(Matcher::String {
            query: QueryMatcher {
                selector: "h1.title".to_string(),
                text_type: TextType::Own,
                clean_with_regex_1: None,
            },
        }),
        if_false: None,
    }
}

#[test]
fn test_case() -> Result<(), Box<dyn Error>> {
    let doc = kuchiki::parse_html()
        .from_utf8()
        .from_file("tests/fragments/test.html")?;

    let cfg = make_cfg();

    cfg.exec(doc.select("html:first-of-type").unwrap());

    Ok(())
}
