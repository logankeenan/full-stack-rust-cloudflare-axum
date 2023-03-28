use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
		pub id: i64,
		pub content: String,
		pub created_at: String,
		pub updated_at: Option<String>,
		pub used_id: String,
}
