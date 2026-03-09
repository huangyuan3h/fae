-- Create allowed_folders table to store folder configurations for tool access
CREATE TABLE IF NOT EXISTS allowed_folders (
    id TEXT PRIMARY KEY NOT NULL,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_base INTEGER DEFAULT 0,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);