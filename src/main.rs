use std::collections::HashMap;

use html_scrape_blocks::model::query_matcher::QueryMatcher;
use html_scrape_blocks::model::scrape_block::Matcher;

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
