-- Enable WAL mode for better concurrency
PRAGMA journal_mode = WAL;

-- Create a settings table to match the daemon's schema
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Create agents table to support provider configuration per agent 
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT 'ollama',
    provider_config_id TEXT,
    model TEXT DEFAULT 'qwen2.5:7b',
    system_prompt TEXT,
    avatar_url TEXT,
    skills_json TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Create messages table to work with agent interactions
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    agent_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Create skills table (to match original daemon schema)
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    enabled INTEGER DEFAULT 1
);

-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    token TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    expires_at INTEGER NOT NULL
);

-- Create channels table
CREATE TABLE IF NOT EXISTS channels (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    topic TEXT NOT NULL DEFAULT '',
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Create channel_users table
CREATE TABLE IF NOT EXISTS channel_users (
    channel_id TEXT NOT NULL,
    user_name TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (channel_id, user_name)
);

-- Create channel_members table
CREATE TABLE IF NOT EXISTS channel_members (
    channel_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (channel_id, agent_id)
);

-- Create channel_messages table
CREATE TABLE IF NOT EXISTS channel_messages (
    id TEXT PRIMARY KEY NOT NULL,
    channel_id TEXT NOT NULL,
    sender_type TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    sender_name TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);