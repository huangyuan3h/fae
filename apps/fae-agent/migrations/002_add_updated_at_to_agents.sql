-- Add updated_at column to agents table
ALTER TABLE agents ADD COLUMN updated_at INTEGER DEFAULT (strftime('%s', 'now'));