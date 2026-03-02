import { Hono } from "hono";
import { z } from "zod";
import type { AppBindings } from "../types";
import { SettingsController } from "../controllers/settings-controller";

const controller = new SettingsController();

const updateOllamaSettingsSchema = z.object({
  baseUrl: z.string().url()
});

const providerSettingsSchema = z.object({
  defaultProvider: z.enum(["ollama", "openai", "google"]),
  ollama: z.object({
    baseUrl: z.string().url()
  }),
  openai: z.object({
    apiKey: z.string().optional().default(""),
    baseUrl: z.string().url().optional().default("https://api.openai.com/v1")
  }),
  google: z.object({
    apiKey: z.string().optional().default(""),
    baseUrl: z
      .string()
      .url()
      .optional()
      .default("https://generativelanguage.googleapis.com/v1beta")
  })
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
