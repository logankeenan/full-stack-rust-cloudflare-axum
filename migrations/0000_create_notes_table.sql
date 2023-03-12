-- Migration number: 0000 	 2023-03-12T11:25:27.605Z
CREATE TABLE notes
(
		id         INTEGER PRIMARY KEY AUTOINCREMENT,
		content    TEXT not null,
		title      TEXT not null,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME,
		used_id    TEXT not null
);
