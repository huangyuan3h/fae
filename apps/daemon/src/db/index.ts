import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { Database as BunDatabase } from "bun:sqlite";
import { drizzle } from "drizzle-orm/bun-sqlite";
import type { Logger } from "pino";
import type { DB } from "./types";
import * as schema from "./schema";

const FALLBACK_SCHEMA = `
CREATE TABLE IF NOT EXISTS agents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  provider TEXT NOT NULL DEFAULT 'ollama',
  model TEXT DEFAULT 'qwen2.5:7b',
  system_prompt TEXT,
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
`;

function readSchemaFile(schemaPath: string): string {
  if (existsSync(schemaPath)) {
    return readFileSync(schemaPath, "utf8");
  }
  return FALLBACK_SCHEMA;
}

function safeExec(rawDb: BunDatabase, sql: string): void {
  try {
    rawDb.exec(sql);
  } catch {
    // Ignore migration statements that are already applied.
  }
}

export function initDatabase(params: {
  dbPath: string;
  logger: Logger;
  schemaPath?: string;
}): DB {
  const schemaPath =
    params.schemaPath ?? path.join(import.meta.dir, "schema.sql");
  const schemaSQL = readSchemaFile(schemaPath);

  const rawDb = new BunDatabase(params.dbPath, { create: true });
  rawDb.exec("PRAGMA journal_mode = WAL;");
  rawDb.exec("PRAGMA busy_timeout = 3000;");
  rawDb.exec(schemaSQL);
  // Backward-compatible schema migrations for older local DB files.
  safeExec(
    rawDb,
    "ALTER TABLE agents ADD COLUMN provider TEXT NOT NULL DEFAULT 'ollama';"
  );
  safeExec(
    rawDb,
    "ALTER TABLE agents ADD COLUMN skills_json TEXT NOT NULL DEFAULT '[]';"
  );

  const orm = drizzle(rawDb, { schema });

  params.logger.info({ dbPath: params.dbPath }, "SQLite initialized");
  return {
    client: rawDb,
    orm,
    close: () => rawDb.close()
  };
}
