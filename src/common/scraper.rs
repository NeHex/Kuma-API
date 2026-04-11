use reqwest::Url;
use scraper::{Html, Selector};

pub fn extract_attr_by_selector(document: &Html, css_selector: &str, attrs: &[&str]) -> Option<String> {
    let selector = Selector::parse(css_selector).ok()?;
    let element = document.select(&selector).next()?;

    attrs
        .iter()
        .find_map(|attr| element.value().attr(attr))
        .map(ToString::to_string)
}

pub fn extract_text_by_selector(document: &Html, css_selector: &str) -> Option<String> {
    let selector = Selector::parse(css_selector).ok()?;
    let text = document.select(&selector).next()?.text().collect::<String>();
    let normalized = normalize_whitespace(&text);

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub fn extract_input_value(document: &Html, css_selector: &str) -> Option<String> {
    extract_attr_by_selector(document, css_selector, &["value"])
}

pub fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn absolutize_url(base: &str, raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        return raw.to_string();
    }

    if raw.starts_with("//") {
        return format!("https:{raw}");
    }

    if let Ok(base_url) = Url::parse(base) {
        if let Ok(joined) = base_url.join(raw) {
            return joined.to_string();
        }
    }

    raw.to_string()
}
