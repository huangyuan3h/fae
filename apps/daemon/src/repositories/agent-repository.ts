import { randomUUID } from "node:crypto";
import { desc, eq } from "drizzle-orm";
import type { DB } from "../db/types";
import { agents, channelMembers, messages, skills } from "../db/schema";
import type { ProviderType } from "../core/llm";
import type { AgentRecord } from "../core/agent";

export interface AgentForChat {
  id: string;
  provider: ProviderType | null;
  model: string;
  system_prompt: string | null;
  skills_json: string | null;
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
    // ignore
  }

  return [];
}

function mapAgentRow(row: {
  id: string;
  name: string;
  provider: string;
  model: string | null;
  systemPrompt: string | null;
  avatarUrl: string | null;
  skillsJson: string;
  createdAt: number | null;
}): AgentRecord {
  return {
    id: row.id,
    name: row.name,
    provider: (row.provider as ProviderType) ?? "ollama",
    model: row.model ?? "qwen2.5:7b",
    system_prompt: row.systemPrompt,
    avatar_url: row.avatarUrl,
    skills: parseSkills(row.skillsJson),
    created_at: row.createdAt ?? 0
  };
}

export class AgentRepository {
  constructor(private readonly db: DB) {}

  list(): AgentRecord[] {
    const rows = this.db.orm
      .select({
        id: agents.id,
        name: agents.name,
        provider: agents.provider,
        model: agents.model,
        systemPrompt: agents.systemPrompt,
        avatarUrl: agents.avatarUrl,
        skillsJson: agents.skillsJson,
        createdAt: agents.createdAt
      })
      .from(agents)
      .orderBy(desc(agents.createdAt))
      .all();

    return rows.map(mapAgentRow);
  }

  create(input: {
    name: string;
    provider?: ProviderType;
    model?: string;
    systemPrompt?: string;
    avatarUrl?: string | null;
    skills?: string[];
  }): AgentRecord {
    const id = randomUUID();
    this.db.orm
      .insert(agents)
      .values({
        id,
        name: input.name,
        provider: input.provider ?? "ollama",
        model: input.model ?? "qwen2.5:7b",
        systemPrompt: input.systemPrompt ?? null,
        avatarUrl: input.avatarUrl ?? null,
        skillsJson: JSON.stringify(input.skills ?? [])
      })
      .run();

    const row = this.db.orm
      .select({
        id: agents.id,
        name: agents.name,
        provider: agents.provider,
        model: agents.model,
        systemPrompt: agents.systemPrompt,
        avatarUrl: agents.avatarUrl,
        skillsJson: agents.skillsJson,
        createdAt: agents.createdAt
      })
      .from(agents)
      .where(eq(agents.id, id))
      .get();

    if (!row) {
      throw new Error("Failed to create agent");
    }

    return mapAgentRow(row);
  }

  update(
    id: string,
    payload: {
      name: string;
      provider: ProviderType;
      model: string;
      systemPrompt?: string | null;
      avatarUrl?: string | null;
      skills?: string[];
    }
  ): boolean {
    const existing = this.db.orm
      .select({ id: agents.id })
      .from(agents)
      .where(eq(agents.id, id))
      .get();

    if (!existing) {
      return false;
    }

    this.db.orm
      .update(agents)
      .set({
        name: payload.name,
        provider: payload.provider,
        model: payload.model,
        systemPrompt: payload.systemPrompt ?? null,
        avatarUrl: payload.avatarUrl ?? null,
        skillsJson: JSON.stringify(payload.skills ?? [])
      })
      .where(eq(agents.id, id))
      .run();

    return true;
  }

  delete(id: string): boolean {
    const existing = this.db.orm
      .select({ id: agents.id })
      .from(agents)
      .where(eq(agents.id, id))
      .get();

    if (!existing) {
      return false;
    }

    this.db.orm.delete(channelMembers).where(eq(channelMembers.agentId, id)).run();
    this.db.orm.delete(agents).where(eq(agents.id, id)).run();
    return true;
  }

  findForChat(id: string): AgentForChat | null {
    const row = this.db.orm
      .select({
        id: agents.id,
        provider: agents.provider,
        model: agents.model,
        system_prompt: agents.systemPrompt,
        skills_json: agents.skillsJson
      })
      .from(agents)
      .where(eq(agents.id, id))
      .get();

    return row
      ? {
          id: row.id,
          provider: row.provider as ProviderType,
          model: row.model ?? "qwen2.5:7b",
          system_prompt: row.system_prompt,
          skills_json: row.skills_json
        }
      : null;
  }

  insertUserMessage(agentId: string, content: string): void {
    this.db.orm
      .insert(messages)
      .values({
        id: randomUUID(),
        agentId,
        role: "user",
        content
      })
      .run();
  }

  insertAssistantMessage(agentId: string, content: string): void {
    this.db.orm
      .insert(messages)
      .values({
        id: randomUUID(),
        agentId,
        role: "assistant",
        content
      })
      .run();
  }

  recentMessages(
    agentId: string,
    limit = 24
  ): Array<{ role: "user" | "assistant"; content: string }> {
    const rows = this.db.orm
      .select({ role: messages.role, content: messages.content })
      .from(messages)
      .where(eq(messages.agentId, agentId))
      .orderBy(desc(messages.createdAt))
      .limit(limit)
      .all();

    return rows
      .reverse()
      .map((row) => ({ role: row.role as "user" | "assistant", content: row.content }));
  }

  enabledSkillIds(): Set<string> {
    const rows = this.db.orm
      .select({ id: skills.id })
      .from(skills)
      .where(eq(skills.enabled, 1))
      .all();
    return new Set(rows.map((row) => row.id));
  }
}
