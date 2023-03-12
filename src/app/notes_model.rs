use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
		id: u64,
		content: String,
		title: String,
		created_at: String,
		updated_at: Option<String>,
		used_id: Uuid,
}
