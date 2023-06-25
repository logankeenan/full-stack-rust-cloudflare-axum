use askama_axum::Template;
use axum::{
		body::Body,
		response::{IntoResponse, Response},
		Form,
		extract::Path,
};
use axum::extract::Query;
use axum_cloudflare_adapter::{worker_route_compat};
use pulldown_cmark::{Event, html, Options, Parser};
use serde::{Deserialize, Serialize};
use crate::{
		app::notes_model::Note,
		app::notes_service::NotesService,
};

use validator::{Validate};
use crate::app::axum_extractors::UserId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteListItem {
		pub id: i64,
		pub title: String,
}

impl NoteListItem {
		pub fn from(note: &Note) -> Self {
				NoteListItem {
						id: note.id,
						title: first_20_chars(&note.content),
				}
		}
}

#[derive(Validate, Debug, Serialize, Deserialize, Clone, Default)]
pub struct NoteForm {
		pub id: Option<i64>,

		#[validate(length(min = 1, message = "Content is too short. It must be at least 1 characters long."))]
		#[validate(length(max = 1000, message = "Content is too long. It must be no more than 1000 characters long."))]
		pub content: String,

		pub content_error: Option<String>,
}

impl NoteForm {
		pub fn from(note: &Note) -> Self {
				NoteForm {
						id: Some(note.id),
						content: note.content.clone(),
						content_error: None,
				}
		}
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
}

fn content_to_markdown(content: &str) -> String {
		let parser = Parser::new(content);
		let mut markdown_output = String::new();

		let filtered_parser = parser.into_iter().filter(|event| {
				!matches!(event, Event::Html(ref html) | Event::Html(ref html) if html.contains("<script"))
		});

		html::push_html(&mut markdown_output, filtered_parser);
		markdown_output
}

fn first_20_chars(markdown_input: &str) -> String {
		let mut options = Options::empty();
		options.insert(Options::ENABLE_STRIKETHROUGH);
		let parser = Parser::new_ext(markdown_input, options);

		let mut plain_text = String::new();
		const LENGTH: usize = 20;

		for event in parser {
				match event {
						Event::Text(text) => plain_text.push_str(&text),
						Event::Code(code) => plain_text.push_str(&code),
						_ => {}
				}
				if plain_text.len() >= LENGTH {
						break;
				}
		}

		plain_text.truncate(LENGTH);
		plain_text
}

#[worker_route_compat]
pub async fn index(
		user_id: UserId,
		note_service: NotesService,
) -> impl IntoResponse {
		let notes = note_service.all_notes_ordered_by_most_recent(user_id.0).await;

		IndexTemplate {
				note_list: notes.iter().map(NoteListItem::from).collect(),
				note_form: NoteForm::default(),
		}
}

#[worker_route_compat]
pub async fn create_note(
		user_id: UserId,
		notes_service: NotesService,
		note_form: Form<NoteForm>,
) -> impl IntoResponse {
		let mut note_form = note_form.0;

		if !note_form.is_valid() {
				let notes = notes_service.all_notes_ordered_by_most_recent(user_id.0).await;

				let index_template = IndexTemplate {
						note_list: notes.iter().map(NoteListItem::from).collect(),
						note_form,
				};

				let html = index_template.render().unwrap();

				Response::builder()
						.status(200)
						.body(html.into())
						.unwrap()
		} else {
				let note = notes_service.create_note(
						note_form.content,
						user_id.0,
				).await;

				let location = format!("/show/{}", note.id);

				Response::builder()
						.header("Location", location)
						.status(303)
						.body(Body::empty())
						.unwrap()
		}
}

#[worker_route_compat]
pub async fn update_note(
		notes_service: NotesService,
		user_id: UserId,
		note_form: Form<NoteForm>,
) -> impl IntoResponse {
		let mut note_form = note_form.0;
		if !note_form.is_valid() {
				let notes = notes_service.all_notes_ordered_by_most_recent(user_id.0).await;

				let index_template = IndexTemplate {
						note_list: notes.iter().map(NoteListItem::from).collect(),
						note_form,
				};

				let html = index_template.render().unwrap();

				Response::builder()
						.status(200)
						.body(html.into())
						.unwrap()
		} else {
				let note = notes_service.update_note(note_form.content, note_form.id.unwrap()).await;
				let location = format!("/show/{}", note.id);

				Response::builder()
						.header("Location", location)
						.status(303)
						.body(Body::empty())
						.unwrap()
		}
}

#[worker_route_compat]
pub async fn show_note(
		notes_service: NotesService,
		Path(id): Path<i64>,
		user_id: UserId,
) -> impl IntoResponse {
		let notes = notes_service.all_notes_ordered_by_most_recent(user_id.0).await;
		let note_by_id = notes.iter().find(|note| note.id == id).cloned();

		if let Some(note) = note_by_id {
				let preview = content_to_markdown(&note.content);

				let show_template = ShowTemplate {
						note_list: notes.iter().map(NoteListItem::from).collect(),
						preview,
						selected_note: note,
				};

				let html: String = show_template.render().unwrap();

				Response::builder()
						.status(200)
						.body(html.into())
						.unwrap()
		} else {
				Response::builder()
						.status(404)
						.body(Body::from("Note not found"))
						.unwrap()
		}
}


#[derive(Template)]
#[template(path = "notes/show.html")]
pub struct ShowTemplate {
		pub note_list: Vec<NoteListItem>,
		pub preview: String,
		pub selected_note: Note,
}


#[worker_route_compat]
pub async fn edit_note(
		notes_service: NotesService,
		Path(id): Path<i64>,
		user_id: UserId,
) -> impl IntoResponse {
		let notes = notes_service.all_notes_ordered_by_most_recent(user_id.0).await;
		let note_by_id = notes.iter().find(|note| note.id == id).cloned();

		if let Some(note) = note_by_id {
				let show_template = EditTemplate {
						note_list: notes.iter().map(NoteListItem::from).collect(),
						note_form: NoteForm::from(&note),
				};

				let html: String = show_template.render().unwrap();

				Response::builder()
						.status(200)
						.body(html.into())
						.unwrap()
		} else {
				Response::builder()
						.status(404)
						.body(Body::from("Note not found"))
						.unwrap()
		}
}

#[derive(Template)]
#[template(path = "notes/edit.html")]
pub struct EditTemplate {
		pub note_list: Vec<NoteListItem>,
		pub note_form: NoteForm,
}


#[derive(Deserialize)]
pub struct SearchQuery {
		search: String,
}

#[worker_route_compat]
pub async fn search_note(
		notes_service: NotesService,
		Query(SearchQuery { search }): Query<SearchQuery>,
		user_id: UserId,
) -> impl IntoResponse {
		let notes = notes_service.all_notes_ordered_by_most_recent(user_id.0).await;

		let filtered_notes: Vec<NoteSearchPreview> = notes
				.iter()
				.filter(|note| note.content.to_lowercase().contains(&search.to_lowercase()))
				.map(|note| {
						let preview = content_to_markdown(&note.content);
						NoteSearchPreview {
								id: note.id,
								preview,
						}
				})
				.collect();

		let search_template = SearchTemplate {
				note_list: notes.iter().map(NoteListItem::from).collect(),
				filtered_notes,
				search,
		};
		let html = search_template.render().unwrap();

		Response::builder()
				.status(200)
				.body(html)
				.unwrap()
}

pub struct NoteSearchPreview {
		pub id: i64,
		pub preview: String
}


#[derive(Template)]
#[template(path = "notes/search.html")]
pub struct SearchTemplate {
		pub note_list: Vec<NoteListItem>,
		pub filtered_notes: Vec<NoteSearchPreview>,
		pub search: String
}

