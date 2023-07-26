use html_scrape_blocks::model::{query_matcher::QueryMatcher, scrape_block::Matcher, scrape_block_error::ScrapeBlockError};

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

    assert!(matches!(result, Err(ScrapeBlockError::ElementExpected(_selector))));

    Ok(())
}
