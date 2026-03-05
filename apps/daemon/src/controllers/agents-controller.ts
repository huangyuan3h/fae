import type { Context } from "hono";
import type { AppBindings } from "../types";
import type { ProviderType } from "../core/llm";
import { AgentsService } from "../services/agents-service";

export class AgentsController {
  list(c: Context<AppBindings>) {
    const service = new AgentsService(c.get("db"));
    return c.json({ ok: true, data: service.list() });
  }

  create(
    c: Context<AppBindings>,
    payload: {
      name: string;
      provider?: ProviderType;
      providerConfigId?: string | null;
      model?: string;
      systemPrompt?: string;
      avatarUrl?: string | null;
      skills?: string[];
    }
  ) {
    const service = new AgentsService(c.get("db"));
    const created = service.create(payload);
    return c.json({ ok: true, data: created }, 201);
  }

  update(
    c: Context<AppBindings>,
    agentId: string,
    payload: {
      name: string;
      provider: ProviderType;
      providerConfigId?: string | null;
      model: string;
      systemPrompt?: string | null;
      avatarUrl?: string | null;
      skills?: string[];
    }
  ) {
    const service = new AgentsService(c.get("db"));
    const updated = service.update(agentId, payload);
    if (!updated) {
      return c.json(
        {
          ok: false,
          error: {
            code: "AGENT_NOT_FOUND",
            message: "Agent does not exist"
          }
        },
        404
      );
    }

    return c.json({ ok: true, data: updated });
  }

  delete(c: Context<AppBindings>, agentId: string) {
    const service = new AgentsService(c.get("db"));
    const ok = service.delete(agentId);

    if (!ok) {
      return c.json(
        {
          ok: false,
          error: {
            code: "AGENT_NOT_FOUND",
            message: "Agent does not exist"
          }
        },
        404
      );
    }

    return c.json({ ok: true, data: { id: agentId } });
  }
}
