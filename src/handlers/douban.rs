use axum::{extract::Path, Json};
use scraper::{Html, Selector};
use serde::Serialize;
use sha2::{Digest, Sha512};

use crate::common::{
    error::{bad_gateway, not_found, ApiError},
    http::{browser_client, with_browser_headers},
    scraper::{absolutize_url, extract_attr_by_selector, extract_input_value, extract_text_by_selector},
};

#[derive(Serialize)]
pub struct DoubanResponse {
    douban: DoubanData,
}

#[derive(Serialize)]
struct DoubanData {
    cover: String,
    title: String,
    years: String,
    score: String,
    desc: String,
    url: String,
}

struct DoubanChallenge {
    tok: String,
    cha: String,
    red: String,
    action: String,
}

pub async fn douban_subject(Path(id): Path<String>) -> Result<Json<DoubanResponse>, ApiError> {
    let target_url = format!("https://movie.douban.com/subject/{id}/");

    let client = browser_client().map_err(|e| bad_gateway(format!("failed to build http client: {e}")))?;
    let first_response = with_browser_headers(client.get(&target_url))
        .send()
        .await
        .map_err(|e| bad_gateway(format!("failed to request douban: {e}")))?;

    let first_url = first_response.url().clone();
    let first_html = first_response
        .text()
        .await
        .map_err(|e| bad_gateway(format!("failed to read douban html: {e}")))?;

    if let Some(douban) = extract_douban_data(&first_html, &id) {
        return Ok(Json(DoubanResponse { douban }));
    }

    let challenge = extract_douban_challenge(&first_html)
        .ok_or_else(|| not_found("cannot parse douban fields or challenge tokens"))?;

    let submit_url = first_url
        .join(&challenge.action)
        .map_err(|e| bad_gateway(format!("failed to build challenge submit url: {e}")))?;

    let cha = challenge.cha.clone();
    let sol = tokio::task::spawn_blocking(move || solve_douban_pow(cha, 4, 5_000_000))
        .await
        .map_err(|e| bad_gateway(format!("failed to solve challenge: {e}")))?
        .ok_or_else(|| bad_gateway("cannot solve douban challenge in limit"))?
        .to_string();

    let second_html = with_browser_headers(client.post(submit_url))
        .header("Origin", first_url.origin().ascii_serialization())
        .header("Referer", first_url.as_str())
        .form(&[
            ("tok", challenge.tok.clone()),
            ("cha", challenge.cha.clone()),
            ("sol", sol),
            ("red", challenge.red.clone()),
        ])
        .send()
        .await
        .map_err(|e| bad_gateway(format!("failed to submit douban challenge: {e}")))?
        .text()
        .await
        .map_err(|e| bad_gateway(format!("failed to read challenge response html: {e}")))?;

    if let Some(douban) = extract_douban_data(&second_html, &id) {
        return Ok(Json(DoubanResponse { douban }));
    }

    let final_html = with_browser_headers(client.get(&challenge.red))
        .send()
        .await
        .map_err(|e| bad_gateway(format!("failed to request redirected douban page: {e}")))?
        .text()
        .await
        .map_err(|e| bad_gateway(format!("failed to read redirected douban html: {e}")))?;

    let douban = extract_douban_data(&final_html, &id)
        .ok_or_else(|| not_found("cannot parse douban fields in redirected html"))?;

    Ok(Json(DoubanResponse { douban }))
}

fn extract_douban_data(html: &str, id: &str) -> Option<DoubanData> {
    let document = Html::parse_document(html);

    let cover_raw = extract_attr_by_selector(&document, "a.nbgnbg img", &["src", "data-src"])?;
    let title = extract_text_by_selector(&document, r#"span[property="v:itemreviewed"]"#)?;
    let years = extract_text_by_selector(&document, "span.year").unwrap_or_default();
    let score = extract_text_by_selector(&document, r#"strong[property="v:average"]"#)
        .unwrap_or_default();
    let desc = extract_text_by_selector(&document, r#"span[property="v:summary"]"#)
        .unwrap_or_default();

    Some(DoubanData {
        cover: absolutize_url("https://movie.douban.com", &cover_raw),
        title,
        years,
        score,
        desc,
        url: format!("https://movie.douban.com/subject/{id}"),
    })
}

fn extract_douban_challenge(html: &str) -> Option<DoubanChallenge> {
    let document = Html::parse_document(html);
    let form_selector = Selector::parse("form#sec").ok()?;
    let form_action = document
        .select(&form_selector)
        .next()
        .and_then(|form| form.value().attr("action"))
        .unwrap_or("/c")
        .to_string();

    Some(DoubanChallenge {
        tok: extract_input_value(&document, "input#tok")?,
        cha: extract_input_value(&document, "input#cha")?,
        red: extract_input_value(&document, "input#red")?,
        action: form_action,
    })
}

fn solve_douban_pow(cha: String, difficulty: usize, max_nonce: u64) -> Option<u64> {
    for nonce in 1..=max_nonce {
        let mut hasher = Sha512::new();
        hasher.update(cha.as_bytes());
        hasher.update(nonce.to_string().as_bytes());
        let digest = hasher.finalize();

        if has_leading_zero_nibbles(&digest, difficulty) {
            return Some(nonce);
        }
    }

    None
}

fn has_leading_zero_nibbles(bytes: &[u8], nibbles: usize) -> bool {
    let full_bytes = nibbles / 2;
    let half_nibble = nibbles % 2;

    if bytes.len() < full_bytes + usize::from(half_nibble > 0) {
        return false;
    }

    if bytes[..full_bytes].iter().any(|b| *b != 0) {
        return false;
    }

    if half_nibble == 1 && (bytes[full_bytes] & 0xF0) != 0 {
        return false;
    }

    true
}
