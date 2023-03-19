use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
		pub id: i64,
		pub content: String,
		pub title: String,
		pub created_at: String,
		pub updated_at: Option<String>,
		pub used_id: Uuid,
}
