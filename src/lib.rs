use axum::{routing::get, routing::post, Router as AxumRouter, middleware};
use axum_cloudflare_adapter::{EnvWrapper, to_axum_request, to_worker_response};
use tower_service::Service;
use worker::{console_log, Env, Request, Response, Date, Result, event};
use crate::app::notes_routes::{index, create_note, update_note};
use crate::app::user_id_middleware::set_user_id_cookie;

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

#[derive(Clone)]
pub struct AppState {
		pub env_wrapper: EnvWrapper
}

impl AppState {
		pub fn new(env_wrapper: EnvWrapper) -> Self {
				AppState {
						env_wrapper,
				}
		}
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

		let state = AppState::new(EnvWrapper::new(_env.clone().into()));

		let mut _router: AxumRouter = AxumRouter::new()
				.route("/", get(index))
				.route("/create", post(create_note))
				.route("/update", post(update_note))
				.layer(middleware::from_fn(set_user_id_cookie))
				.with_state(state);

		let axum_request = to_axum_request(req).await.unwrap();
		let axum_response = _router.call(axum_request).await.unwrap();
		let response = to_worker_response(axum_response).await.unwrap();

		Ok(response)
}
