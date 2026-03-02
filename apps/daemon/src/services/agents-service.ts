import type { ProviderType } from "../core/llm";
import { AgentRepository } from "../repositories/agent-repository";
import type { DB } from "../db/types";

export class AgentsService {
  private readonly repo: AgentRepository;

  constructor(db: DB) {
    this.repo = new AgentRepository(db);
  }

  list() {
    return this.repo.list();
  }

  create(input: {
    name: string;
    provider?: ProviderType;
    model?: string;
    systemPrompt?: string;
    avatarUrl?: string | null;
    skills?: string[];
  }) {
    return this.repo.create(input);
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
  ) {
    const ok = this.repo.update(id, payload);
    if (!ok) {
      return null;
    }
    return this.repo.list().find((agent) => agent.id === id) ?? null;
  }

  delete(id: string): boolean {
    return this.repo.delete(id);
  }
}
