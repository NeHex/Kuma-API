use axum::{extract::Path, Json};
use reqwest::header::LOCATION;
use serde::{Deserialize, Deserializer, Serialize};

use crate::common::{
    error::{bad_gateway, not_found, ApiError},
    http::{browser_client, browser_client_no_redirect, with_browser_headers},
};

#[derive(Serialize)]
pub struct Music163Response {
    music163: Music163Data,
}

#[derive(Serialize)]
struct Music163Data {
    stream_url: String,
    info: NeteaseInfo,
}

#[derive(Debug, Deserialize, Serialize)]
struct NeteaseInfo {
    #[serde(deserialize_with = "deserialize_id")]
    id: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    album: String,
    #[serde(default)]
    cover: String,
    #[serde(default)]
    lyric: String,
    #[serde(default)]
    sub_lyric: String,
    #[serde(default)]
    link: String,
    #[serde(default)]
    served: bool,
    #[serde(default)]
    cached: bool,
    #[serde(default)]
    remaining: i64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Number(u64),
}

fn deserialize_id<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(value) => value.parse::<u64>().map_err(serde::de::Error::custom),
        StringOrNumber::Number(value) => Ok(value),
    }
}

pub async fn music_163(Path(id): Path<String>) -> Result<Json<Music163Response>, ApiError> {
    let redirect_api = format!("https://node.api.xfabe.com/api/wangyi/music?id={id}");
    let info_api = format!("https://api.paugram.com/netease/?id={id}");

    let redirect_client = browser_client_no_redirect()
        .map_err(|e| bad_gateway(format!("failed to build redirect client: {e}")))?;
    let redirect_response = with_browser_headers(redirect_client.get(&redirect_api))
        .send()
        .await
        .map_err(|e| bad_gateway(format!("failed to request 163 music redirect api: {e}")))?;

    let redirect_from_header = redirect_response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let redirect_body = redirect_response.text().await.map_err(|e| {
        bad_gateway(format!(
            "failed to read 163 music redirect response body: {e}"
        ))
    })?;

    let stream_url = redirect_from_header
        .or_else(|| extract_redirect_url_from_body(&redirect_body))
        .ok_or_else(|| not_found("cannot parse 163 music redirect url"))?;

    let client =
        browser_client().map_err(|e| bad_gateway(format!("failed to build http client: {e}")))?;
    let info = with_browser_headers(client.get(&info_api))
        .send()
        .await
        .map_err(|e| bad_gateway(format!("failed to request 163 music info api: {e}")))?
        .json::<NeteaseInfo>()
        .await
        .map_err(|e| bad_gateway(format!("failed to parse 163 music info json: {e}")))?;

    Ok(Json(Music163Response {
        music163: Music163Data { stream_url, info },
    }))
}

fn extract_redirect_url_from_body(body: &str) -> Option<String> {
    let marker = "Redirecting to ";
    let idx = body.find(marker)?;
    let target = body[idx + marker.len()..].trim();

    if target.is_empty() {
        None
    } else {
        Some(target.to_string())
    }
}
