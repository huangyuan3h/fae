import { Hono } from "hono";
import { z } from "zod";
import type { AppBindings } from "../types";
import { AgentsController } from "../controllers/agents-controller";

const controller = new AgentsController();

const createAgentSchema = z.object({
  name: z.string().min(1),
  provider: z.enum(["ollama", "openai", "google"]).optional(),
  model: z.string().min(1).optional(),
  systemPrompt: z.string().optional(),
  avatarUrl: z.string().nullable().optional(),
  skills: z.array(z.string().min(1)).optional()
});

const updateAgentSchema = z.object({
  name: z.string().min(1),
  provider: z.enum(["ollama", "openai", "google"]),
  model: z.string().min(1),
  systemPrompt: z.string().nullable().optional(),
  avatarUrl: z.string().nullable().optional(),
  skills: z.array(z.string().min(1)).optional()
});

export const agentRoutes = new Hono<AppBindings>();

agentRoutes.get("/", (c) => controller.list(c));

agentRoutes.post("/", async (c) => {
  const payload = createAgentSchema.parse(await c.req.json());
  return controller.create(c, payload);
});

agentRoutes.put("/:id", async (c) => {
  const payload = updateAgentSchema.parse(await c.req.json());
  return controller.update(c, c.req.param("id"), payload);
});

agentRoutes.delete("/:id", (c) => controller.delete(c, c.req.param("id")));
