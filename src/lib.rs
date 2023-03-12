use axum::{
		routing::get,
		Router as AxumRouter,
};
use axum_cloudflare_adapter::{to_axum_request, to_worker_response};
use tower_service::Service;
use worker::{console_log, Env, Request, Response, Date, Result, event};
use crate::app::notes_routes::index;

mod utils;
mod app;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

		let mut _router: AxumRouter = AxumRouter::new()
				.route("/", get(index));

		let axum_request = to_axum_request(req).await.unwrap();
		let axum_response = _router.call(axum_request).await.unwrap();
		let response = to_worker_response(axum_response).await.unwrap();

		Ok(response)
}
