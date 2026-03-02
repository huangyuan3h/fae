CREATE TABLE IF NOT EXISTS agents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  provider TEXT NOT NULL DEFAULT 'ollama',
  model TEXT DEFAULT 'qwen2.5:7b',
  system_prompt TEXT,
  avatar_url TEXT,
  skills_json TEXT NOT NULL DEFAULT '[]',
  created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS skills (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  enabled INTEGER DEFAULT 1
);

CREATE TABLE IF NOT EXISTS sessions (
  token TEXT PRIMARY KEY,
  created_at INTEGER DEFAULT (strftime('%s', 'now')),
  expires_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_messages_agent_created_at
ON messages (agent_id, created_at);

CREATE TABLE IF NOT EXISTS channels (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  topic TEXT NOT NULL DEFAULT '',
  created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS channel_users (
  channel_id TEXT NOT NULL,
  user_name TEXT NOT NULL,
  created_at INTEGER DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (channel_id, user_name)
);

CREATE TABLE IF NOT EXISTS channel_members (
  channel_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  created_at INTEGER DEFAULT (strftime('%s', 'now')),
  PRIMARY KEY (channel_id, agent_id)
);

CREATE TABLE IF NOT EXISTS channel_messages (
  id TEXT PRIMARY KEY,
  channel_id TEXT NOT NULL,
  sender_type TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  sender_name TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_channel_messages_channel_created_at
ON channel_messages (channel_id, created_at);
