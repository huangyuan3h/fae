import { randomUUID } from "node:crypto";
import type { DB } from "../db/types";

export interface AgentRecord {
  id: string;
  name: string;
  model: string;
  system_prompt: string | null;
  created_at: number;
}

export function listAgents(db: DB): AgentRecord[] {
  return db
    .prepare(
      "SELECT id, name, model, system_prompt, created_at FROM agents ORDER BY created_at DESC"
    )
    .all() as AgentRecord[];
}

export function createAgent(
  db: DB,
  input: { name: string; model?: string; systemPrompt?: string }
): AgentRecord {
  const id = randomUUID();
  const model = input.model ?? "qwen2.5:7b";
  const systemPrompt = input.systemPrompt ?? null;

  db.prepare(
    "INSERT INTO agents (id, name, model, system_prompt) VALUES (?, ?, ?, ?)"
  ).run(id, input.name, model, systemPrompt);

  return db
    .prepare(
      "SELECT id, name, model, system_prompt, created_at FROM agents WHERE id = ?"
    )
    .get(id) as AgentRecord;
}
