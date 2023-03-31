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

		pub async fn all_notes_ordered_by_most_recent(&self, user_id: Uuid) -> Vec<Note> {
				let d1 = self.env_wrapper.env.d1("DB").unwrap();
				let prepared_statement = d1.prepare("SELECT * FROM notes where user_id = ? order by created_at desc;")
						.bind(&[user_id.to_string().into()])
						.expect("failed to bind query params");
				let result: worker::Result<D1Result> = prepared_statement.all().await;
				match result {
						Ok(result) => {
								result.results().unwrap()
						}
						Err(e) => {
								console_log!("e: {}", e);
								console_log!("an error: all_notes_ordered_by_most_recent");
								vec![]
						}
				}
		}

		pub async fn update_note(
				&self,
				content: String,
				note_id: i64,
		) -> Note {
				let create_query = "update notes set content = ?, updated_at = CURRENT_TIMESTAMP where id = ? returning *;";

				let d1: worker::d1::Database = self.env_wrapper.env.d1("DB").unwrap();
				let query = d1.prepare(
						create_query,
				).bind(&[content.into(), note_id.to_string().into()])
						.expect("failed to bind query params");

				query
						.first::<Note>(None)
						.await
						.expect("failed to insert note")
						.unwrap()
		}

		pub async fn create_note(
				&self,
				content: String,
				user_id: Uuid,
		) -> Note {
				let create_query = "INSERT INTO notes (content, user_id) VALUES (?, ?) returning *;";

				let d1: worker::d1::Database = self.env_wrapper.env.d1("DB").unwrap();
				let query = d1.prepare(
						create_query,
				).bind(&[content.into(), user_id.to_string().into()])
						.expect("failed to bind query params");

				query
						.first::<Note>(None)
						.await
						.expect("failed to insert note")
						.unwrap()
		}

		pub async fn note_by_id(&self, id: i64, user_id: Uuid) -> Option<Note> {
				let notes = self.all_notes_ordered_by_most_recent(user_id).await;

				notes.iter().find(|note| note.id == id).cloned()
		}

		pub async fn delete_notes_old_than_15_minutes(&self) {
			let query = "DELETE FROM notes WHERE strftime('%s', CURRENT_TIMESTAMP) - strftime('%s', created_at) > 15 * 60;";

				let d1: worker::d1::Database = self.env_wrapper.env.d1("DB").unwrap();

				d1.prepare(query).run().await.expect("failed to execute delete statement");
		}
}
