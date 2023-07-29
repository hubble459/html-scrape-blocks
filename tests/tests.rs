use html_scrape_blocks::model::{
    query_matcher::QueryMatcher, scrape_block::Matcher, scrape_block_error::ScrapeBlockError,
};

#[cfg(test)]
fn get_test_doc(
) -> Result<kuchiki::iter::Elements<kuchiki::iter::Descendants>, Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_module("html_scrape_blocks", log::LevelFilter::Debug)
        .is_test(true)
        .try_init()
        .ok();

    use kuchiki::traits::{NodeIterator, TendrilSink};
    Ok(kuchiki::parse_html()
        .from_utf8()
        .from_file("tests/fragments/test.html")?
        .descendants()
        .elements())
}

#[test]
fn test_string_matcher() -> Result<(), Box<dyn std::error::Error>> {
    let matcher = Matcher::String {
        query: QueryMatcher {
            selector: String::from("h1.title"),
            ..Default::default()
        },
    };

    let doc = get_test_doc()?;
    let result = matcher.exec_string(doc)?;

    assert_eq!(result, "Test Title");

    Ok(())
}

#[test]
fn test_string_array_matcher() -> Result<(), Box<dyn std::error::Error>> {
    let doc = get_test_doc()?;

    let matcher = Matcher::StringArray {
        query: QueryMatcher {
            selector: String::from("ul > li"),
            ..Default::default()
        },
        split_regex: None,
    };

    let result = matcher.exec_string_array(doc.clone())?;
    assert_eq!(result, &["One", "Two", "Three"]);

    let matcher = Matcher::StringArray {
        query: QueryMatcher {
            selector: String::from("h1.title"),
            ..Default::default()
        },
        split_regex: Some(String::from(" ")),
    };

    let result = matcher.exec_string_array(doc)?;
    assert_eq!(result, &["Test", "Title"]);

    Ok(())
}

#[test]
fn test_default_matcher() -> Result<(), Box<dyn std::error::Error>> {
    let matcher = Matcher::String {
        query: QueryMatcher {
            selector: String::from("h1.not-found"),
            default: Some(String::from("Default Value")),
            ..Default::default()
        },
    };

    let doc = get_test_doc()?;
    let result = matcher.exec_string(doc)?;

    assert_eq!(result, "Default Value");

    Ok(())
}

#[test]
fn test_not_found_matcher() -> Result<(), Box<dyn std::error::Error>> {
    let matcher = Matcher::String {
        query: QueryMatcher {
            selector: String::from("h1.not-found"),
            ..Default::default()
        },
    };

    let doc = get_test_doc()?;
    let result = matcher.exec_string(doc);

    assert!(matches!(
        result,
        Err(ScrapeBlockError::ElementExpected(_selector))
    ));

    Ok(())
}

#[test]
fn test_array() -> Result<(), Box<dyn std::error::Error>> {
    let matcher = Matcher::Array {
        query: QueryMatcher {
            selector: "ul li".to_string(),
            ..Default::default()
        },
        each: Box::new(Matcher::String {
            query: QueryMatcher {
                selector: String::from("*"),
                ..Default::default()
            },
        }),
    };

    let doc = get_test_doc()?;
    let result = matcher.exec_array::<_, String>(doc)?;

    assert_eq!(result, &["One", "Two", "Three"]);

    Ok(())
}
