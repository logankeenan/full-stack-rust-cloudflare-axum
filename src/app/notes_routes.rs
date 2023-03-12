use axum::response::{Html, IntoResponse};
use axum_cloudflare_adapter::{worker_route_compat};

#[worker_route_compat]
pub async fn index() -> impl IntoResponse {
	Html("hello world!")
}
