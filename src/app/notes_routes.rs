use askama_axum::Template;
use axum::extract::State;
use axum::response::IntoResponse;
use axum_cloudflare_adapter::{worker_route_compat};
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use crate::app::notes_model::Note;
use crate::app::notes_service::NotesService;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteListItem {
		pub id: i64,
		pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NoteForm {
		pub id: Option<i64>,
		pub content: String,
}

#[derive(Template)]
#[template(path = "notes/index.html")]
pub struct IndexTemplate {
		pub note_list: Vec<NoteListItem>,
		pub note_form: NoteForm,
		pub preview: Option<String>,
}


fn create_note_form_from_first_note_or_empty(notes: &[Note]) -> NoteForm {
		match notes.first() {
				Some(first_note) => NoteForm {
						id: Some(first_note.id),
						content: first_note.content.clone(),
				},
				None => NoteForm::default(),
		}
}

pub fn map_notes_to_note_list_items(notes: &Vec<Note>) -> Vec<NoteListItem> {
		notes.into_iter()
				.map(|note| NoteListItem {
						id: note.id,
						title: note.title.clone(),
				})
				.collect()
}

fn content_to_markdown(content: &str) -> String {
		let parser = Parser::new(content);
		let mut markdown_output = String::new();
		html::push_html(&mut markdown_output, parser);
		markdown_output
}

fn preview_markdown(notes: &Vec<Note>) -> Option<String> {
		notes
				.first()
				.map(|note| content_to_markdown(&note.content))
}

#[worker_route_compat]
pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
		let service = NotesService::new(state.env_wrapper);
		let notes = service.all_notes_ordered_by_most_recent().await;

		let preview = preview_markdown(&notes);

		let note_form = create_note_form_from_first_note_or_empty(&notes);

		IndexTemplate { note_list: map_notes_to_note_list_items(&notes), note_form, preview }
}
