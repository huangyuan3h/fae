import { Hono } from "hono";
import { z } from "zod";
import type { AppBindings } from "../types";

const OLLAMA_BASE_URL_KEY = "ollama.baseUrl";

const updateOllamaSettingsSchema = z.object({
  baseUrl: z.string().url()
});

export const settingsRoutes = new Hono<AppBindings>();

settingsRoutes.get("/ollama", (c) => {
  const row = c
    .get("db")
    .prepare<{ value: string }>("SELECT value FROM settings WHERE key = ?")
    .get(OLLAMA_BASE_URL_KEY);

  return c.json({
    ok: true,
    data: {
      baseUrl: row?.value ?? "http://127.0.0.1:11434"
    }
  });
});

settingsRoutes.put("/ollama", async (c) => {
  const payload = updateOllamaSettingsSchema.parse(await c.req.json());
  const now = Math.floor(Date.now() / 1000);

  c.get("db")
    .prepare(
      "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"
    )
    .run(OLLAMA_BASE_URL_KEY, payload.baseUrl, now);

  return c.json({
    ok: true,
    data: {
      baseUrl: payload.baseUrl
    }
  });
});
