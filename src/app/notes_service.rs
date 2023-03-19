use worker::{console_log, D1Result};
use axum_cloudflare_adapter::EnvWrapper;
use crate::app::notes_model::Note;

pub struct NotesService {
		pub env_wrapper: EnvWrapper,
}


impl NotesService {
		pub fn new(env_wrapper: EnvWrapper) -> Self {
				NotesService {
						env_wrapper,
				}
		}

		pub async fn all_notes_ordered_by_most_recent(&self) -> Vec<Note> {
				let d1 = self.env_wrapper.env.d1("DB").unwrap();
				let prepared_statement = d1.prepare("SELECT * FROM notes");
				let result: worker::Result<D1Result> = prepared_statement.all().await;
				match result {
						Ok(result) => {
								result.results().unwrap()
						}
						Err(_) => {
								console_log!("an error: all_notes_ordered_by_most_recent");
								vec![]
						}
				}
		}
}
