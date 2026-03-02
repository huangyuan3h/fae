import type { Context } from "hono";
import type { AppBindings } from "../types";
import { SettingsService } from "../services/settings-service";
import type { ProviderSettingsDTO } from "../repositories/settings-repository";

export class SettingsController {
  getOllama(c: Context<AppBindings>) {
    const service = new SettingsService(c.get("db"));
    return c.json({ ok: true, data: service.getOllama() });
  }

  saveOllama(c: Context<AppBindings>, baseUrl: string) {
    const service = new SettingsService(c.get("db"));
    return c.json({ ok: true, data: service.saveOllama(baseUrl) });
  }

  getProviders(c: Context<AppBindings>) {
    const service = new SettingsService(c.get("db"));
    return c.json({ ok: true, data: service.getProviders() });
  }

  saveProviders(c: Context<AppBindings>, payload: ProviderSettingsDTO) {
    const service = new SettingsService(c.get("db"));
    return c.json({ ok: true, data: service.saveProviders(payload) });
  }
}
