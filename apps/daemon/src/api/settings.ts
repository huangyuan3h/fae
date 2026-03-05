import { Hono } from "hono";
import { z } from "zod";
import type { AppBindings } from "../types";
import { SettingsController } from "../controllers/settings-controller";

const controller = new SettingsController();

const updateOllamaSettingsSchema = z.object({
  baseUrl: z.string().url()
});

const providerConfigSchema = z.object({
  id: z.string().min(1),
  name: z.string().min(1),
  type: z.enum(["ollama", "openai", "google", "alibaba"]),
  apiKey: z.string().optional().default(""),
  baseUrl: z.string().url(),
  modelId: z.string().optional().default(""),
  enabled: z.boolean().optional().default(true)
});

const providerSettingsSchema = z.object({
  providerConfigs: z.array(providerConfigSchema).default([])
});

export const settingsRoutes = new Hono<AppBindings>();

settingsRoutes.get("/ollama", (c) => controller.getOllama(c));

settingsRoutes.put("/ollama", async (c) => {
  const payload = updateOllamaSettingsSchema.parse(await c.req.json());
  return controller.saveOllama(c, payload.baseUrl);
});

settingsRoutes.get("/providers", (c) => controller.getProviders(c));

settingsRoutes.put("/providers", async (c) => {
  const payload = providerSettingsSchema.parse(await c.req.json());
  return controller.saveProviders(c, payload);
});
