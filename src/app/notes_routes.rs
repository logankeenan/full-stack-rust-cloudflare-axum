use askama_axum::Template;
use axum::{
		body::Body,
		extract::State,
		extract::Query,
		response::{IntoResponse, Response},
		Form
};
use axum_cloudflare_adapter::{worker_route_compat};
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use crate::{
		app::notes_model::Note,
		app::notes_service::NotesService,
		AppState
};
use uuid::Uuid;

use validator::{Validate};
use worker::console_log;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteListItem {
		pub id: i64,
		pub title: String,
}

#[derive(Validate, Debug, Serialize, Deserialize, Clone, Default)]
pub struct NoteForm {
		pub id: Option<i64>,

		// TODO: probably don't need a min length
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


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexQueryParams {
		id: Option<i64>,
}

#[worker_route_compat]
pub async fn index(
		Query(query_params): Query<IndexQueryParams>,
		State(state): State<AppState>,
) -> impl IntoResponse {
		match query_params.id {
				None => {}
				Some(id) => {
						console_log!("id: {}, ", id);
				}
		}

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

		let service = NotesService::new(state.env_wrapper);
		if !note_form.is_valid() {
				let notes = service.all_notes_ordered_by_most_recent().await;
				let preview = content_to_markdown(&note_form.content);

				let index_template = IndexTemplate {
						note_list: map_notes_to_note_list_items(&notes),
						note_form,
						preview: Some(preview),
				};

				let html = index_template.render().unwrap();

				Response::builder()
						.status(200)
						.body(html.into())
						.unwrap()
		} else {
				let note = service.create_note(
						note_form.content,
						"title".to_string(),
						Uuid::new_v4(),
				).await;

				let location = format!("/?id={}", note.id);

				Response::builder()
						.header("Location", location)
						.status(303)
						.body(Body::empty())
						.unwrap()
		}
}

