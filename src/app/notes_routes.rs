use askama_axum::Template;
use axum::extract::State;
use axum::response::{IntoResponse};
use axum_cloudflare_adapter::{worker_route_compat};
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use crate::app::notes_model::Note;
use crate::app::notes_service::NotesService;
use crate::AppState;
use axum::Form;
use axum::response::Html;

use validator::{Validate};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteListItem {
		pub id: i64,
		pub title: String,
}

#[derive(Validate, Debug, Serialize, Deserialize, Clone, Default)]
pub struct NoteForm {
		pub id: Option<i64>,

		#[validate(length(min = 10, message = "Content is too short. It must be at least 10 characters long."))]
		#[validate(length(max = 1000, message = "Content is too long. It must be no more than 1000 characters long."))]
		pub content: String,

		pub content_error: Option<String>,
}

impl NoteForm {
		pub fn is_valid(&mut self) -> bool {
				let result = self.validate();
				if result.is_err() {
						self.content_error = Some(result.unwrap_err().to_string());
						false
				} else {
						self.content_error = None;
						true
				}
		}
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
						content_error: None,
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


#[worker_route_compat]
pub async fn new_note(
		State(state): State<AppState>,
		note_form: Form<NoteForm>,
) -> impl IntoResponse {
		let mut note_form = note_form.0;

		if !note_form.is_valid() {
				let service = NotesService::new(state.env_wrapper);
				let notes = service.all_notes_ordered_by_most_recent().await;
				let preview = content_to_markdown(&note_form.content);

				let index_template = IndexTemplate {
						note_list: map_notes_to_note_list_items(&notes),
						note_form,
						preview: Some(preview),
				};

				let html = index_template.render().unwrap();
				Html(html)
		} else {
				Html("success!".to_string())
		}
}

