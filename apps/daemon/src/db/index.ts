import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { Database as BunDatabase } from "bun:sqlite";
import type { Logger } from "pino";
import type { DB } from "./types";

const FALLBACK_SCHEMA = `
CREATE TABLE IF NOT EXISTS agents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  model TEXT DEFAULT 'qwen2.5:7b',
  system_prompt TEXT,
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
`;

function readSchemaFile(schemaPath: string): string {
  if (existsSync(schemaPath)) {
    return readFileSync(schemaPath, "utf8");
  }
  return FALLBACK_SCHEMA;
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

  params.logger.info({ dbPath: params.dbPath }, "SQLite initialized");
  return {
    exec: (sql) => rawDb.exec(sql),
    prepare: <Row = unknown>(sql: string) => {
      const stmt = rawDb.query(sql);
      return {
        run: (...paramsList) => stmt.run(...(paramsList as never[])),
        get: (...paramsList) => stmt.get(...(paramsList as never[])) as Row | null,
        all: (...paramsList) => stmt.all(...(paramsList as never[])) as Row[]
      };
    },
    close: () => rawDb.close()
  };
}
