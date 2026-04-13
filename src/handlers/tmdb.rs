use axum::{extract::Path, Json};
use scraper::Html;
use serde::Serialize;

use crate::common::{
    error::{bad_gateway, not_found, ApiError},
    http::{browser_client, with_browser_headers},
    scraper::{absolutize_url, extract_attr_by_selector, extract_text_by_selector},
};

#[derive(Serialize)]
pub struct TmdbResponse {
    tmdb: TmdbData,
}

#[derive(Serialize)]
struct TmdbData {
    cover: String,
    title: String,
    years: String,
    desc: String,
    url: String,
}

pub async fn tmdb_movie(Path(id): Path<String>) -> Result<Json<TmdbResponse>, ApiError> {
    let target_url = format!("https://www.themoviedb.org/movie/{id}");

    let client =
        browser_client().map_err(|e| bad_gateway(format!("failed to build http client: {e}")))?;
    let html = with_browser_headers(client.get(&target_url))
        .send()
        .await
        .map_err(|e| bad_gateway(format!("failed to request tmdb: {e}")))?
        .text()
        .await
        .map_err(|e| bad_gateway(format!("failed to read tmdb html: {e}")))?;

    let tmdb = extract_tmdb_data(&html, &id)
        .ok_or_else(|| not_found("cannot parse tmdb fields from html"))?;

    Ok(Json(TmdbResponse { tmdb }))
}

fn extract_tmdb_data(html: &str, id: &str) -> Option<TmdbData> {
    let document = Html::parse_document(html);

    let cover_raw = extract_attr_by_selector(&document, "img.poster.w-full", &["src", "data-src"])
        .or_else(|| {
            extract_attr_by_selector(&document, ".poster.w-full img", &["src", "data-src"])
        })?;

    let title = extract_text_by_selector(&document, "div.title.ott_false h2 a")?;
    let years = extract_text_by_selector(&document, "span.tag.release_date").unwrap_or_default();
    let desc = extract_text_by_selector(&document, "div.overview p").unwrap_or_default();

    Some(TmdbData {
        cover: absolutize_url("https://www.themoviedb.org", &cover_raw),
        title,
        years,
        desc,
        url: format!("https://www.themoviedb.org/movie/{id}"),
    })
}
