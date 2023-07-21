use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
enum TextType {
    #[default]
    Own,
    All(),
    Attribute(),
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct QueryMatcher {
    selector: String,
    text_type: TextType,
    clean_with_regex_1: Option<String>,
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
