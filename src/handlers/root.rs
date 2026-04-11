use axum::Json;

pub async fn hello() -> Json<Vec<&'static str>> {
    Json(vec!["hello,welcome to Kuma API; Visite: https://github.com/nehex/kuma-api"])
}
