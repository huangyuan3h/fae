import { Hono } from "hono";
import { z } from "zod";
import type { AppBindings } from "../types";
import { createAgent, listAgents } from "../core/agent";

const createAgentSchema = z.object({
  name: z.string().min(1),
  model: z.string().min(1).optional(),
  systemPrompt: z.string().optional()
});

export const agentRoutes = new Hono<AppBindings>();

agentRoutes.get("/", (c) => {
  const agents = listAgents(c.get("db"));
  return c.json({ ok: true, data: agents });
});

agentRoutes.post("/", async (c) => {
  const payload = createAgentSchema.parse(await c.req.json());
  const agent = createAgent(c.get("db"), payload);
  return c.json({ ok: true, data: agent }, 201);
});
