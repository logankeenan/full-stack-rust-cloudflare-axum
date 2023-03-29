use axum::{
		http::{Request, header::{COOKIE, SET_COOKIE, HeaderValue}},
		middleware::Next,
		response::Response
};
use cookie::{Cookie, CookieJar};
use uuid::Uuid;

pub async fn set_user_id_cookie<B>(request: Request<B>, next: Next<B>,) -> Response {
		let headers = request.headers();
		let mut cookie_jar = CookieJar::new();

		if let Some(cookie_header) = headers.get(COOKIE) {
				let cookie_str = cookie_header.to_str().unwrap_or_default();

				for cookie in cookie_str.split(';').map(|c| c.trim()) {
						if let Ok(parsed_cookie) = Cookie::parse(cookie.to_string()) {
								cookie_jar.add_original(parsed_cookie);
						}
				}
		}

		let mut response = next.run(request).await;

		if !cookie_jar.get("user_id").is_some() {
				let user_id_cookie = Cookie::build("user_id", Uuid::new_v4().to_string())
						.path("/")
						.http_only(true)
						.finish();

				response.headers_mut().append(
						SET_COOKIE,
						HeaderValue::from_str(&user_id_cookie.to_string()).unwrap(),
				);
		}

		response
}
