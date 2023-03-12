use askama_axum::Template;
use axum_cloudflare_adapter::{worker_route_compat};

#[derive(Template)]
#[template(path = "notes/index.html")]
pub struct IndexTemplate {

}

#[worker_route_compat]
pub async fn index() -> IndexTemplate {
		IndexTemplate {}
}
