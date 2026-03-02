import { randomUUID } from "node:crypto";
import { desc, eq } from "drizzle-orm";
import type { DB } from "../db/types";
import { agents } from "../db/schema";

export interface AgentRecord {
  id: string;
  name: string;
  provider: "ollama" | "openai" | "google";
  model: string;
  system_prompt: string | null;
  skills: string[];
  created_at: number;
}

interface AgentRow {
  id: string;
  name: string;
  provider: "ollama" | "openai" | "google";
  model: string;
  system_prompt: string | null;
  skills_json: string;
  created_at: number;
}

function parseSkills(skillsJson: string | null | undefined): string[] {
  if (!skillsJson) {
    return [];
  }

  try {
    const parsed = JSON.parse(skillsJson) as unknown;
    if (Array.isArray(parsed)) {
      return parsed.filter((entry): entry is string => typeof entry === "string");
    }
  } catch {
    // fall through
  }

  return [];
}

function mapRow(row: AgentRow): AgentRecord {
  return {
    id: row.id,
    name: row.name,
    provider: row.provider,
    model: row.model,
    system_prompt: row.system_prompt,
    skills: parseSkills(row.skills_json),
    created_at: row.created_at
  };
}

export function listAgents(db: DB): AgentRecord[] {
  const rows = db.orm
    .select({
      id: agents.id,
      name: agents.name,
      provider: agents.provider,
      model: agents.model,
      system_prompt: agents.systemPrompt,
      skills_json: agents.skillsJson,
      created_at: agents.createdAt
    })
    .from(agents)
    .orderBy(desc(agents.createdAt))
    .all() as AgentRow[];

  return rows.map(mapRow);
}

export function createAgent(
  db: DB,
  input: {
    name: string;
    provider?: "ollama" | "openai" | "google";
    model?: string;
    systemPrompt?: string;
    skills?: string[];
  }
): AgentRecord {
  const id = randomUUID();
  const provider = input.provider ?? "ollama";
  const model = input.model ?? "qwen2.5:7b";
  const systemPrompt = input.systemPrompt ?? null;
  const skillsJson = JSON.stringify(input.skills ?? []);

  db.orm
    .insert(agents)
    .values({
      id,
      name: input.name,
      provider,
      model,
      systemPrompt,
      skillsJson
    })
    .run();

  const row = db.orm
    .select({
      id: agents.id,
      name: agents.name,
      provider: agents.provider,
      model: agents.model,
      system_prompt: agents.systemPrompt,
      skills_json: agents.skillsJson,
      created_at: agents.createdAt
    })
    .from(agents)
    .where(eq(agents.id, id))
    .get() as AgentRow;

  return mapRow(row);
}
