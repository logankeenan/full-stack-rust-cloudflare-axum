use worker::{console_log, D1Result};
use axum_cloudflare_adapter::EnvWrapper;
use uuid::Uuid;
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
				let prepared_statement = d1.prepare("SELECT * FROM notes order by create_at desc");
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

		pub async fn create_note(
				&self,
				content: String,
				title: String,
				user_id: Uuid,
		) -> Note {
				let create_query = "INSERT INTO notes (content, title, used_id) VALUES (?, ?, ?) returning *;";

				let d1: worker::d1::Database = self.env_wrapper.env.d1("DB").unwrap();
				let query = d1.prepare(
						create_query,
				).bind(&[content.into(), title.into(), user_id.to_string().into()])
						.expect("failed to bind query params");

				query
						.first::<Note>(None)
						.await
						.expect("failed to insert note")
						.unwrap()
		}
}
