import { eq } from "drizzle-orm";
import type { DB } from "../db/types";
import type { ProviderType } from "../core/llm";
import { settings } from "../db/schema";

const OLLAMA_BASE_URL_KEY = "provider.ollama.baseUrl";
const OPENAI_API_KEY = "provider.openai.apiKey";
const OPENAI_BASE_URL = "provider.openai.baseUrl";
const GOOGLE_API_KEY = "provider.google.apiKey";
const GOOGLE_BASE_URL = "provider.google.baseUrl";
const DEFAULT_PROVIDER = "provider.default";

export interface ProviderSettingsDTO {
  defaultProvider: ProviderType;
  ollama: { baseUrl: string };
  openai: { apiKey: string; baseUrl: string };
  google: { apiKey: string; baseUrl: string };
}

export class SettingsRepository {
  constructor(private readonly db: DB) {}

  getSetting(key: string): string | null {
    const row = this.db.orm
      .select({ value: settings.value })
      .from(settings)
      .where(eq(settings.key, key))
      .get();

    return row?.value ?? null;
  }

  upsertSetting(key: string, value: string): void {
    const now = Math.floor(Date.now() / 1000);
    this.db.orm
      .insert(settings)
      .values({ key, value, updatedAt: now })
      .onConflictDoUpdate({
        target: settings.key,
        set: {
          value,
          updatedAt: now
        }
      })
      .run();
  }

  getOllamaBaseUrl(): string {
    return this.getSetting(OLLAMA_BASE_URL_KEY) ?? "http://127.0.0.1:11434";
  }

  saveOllamaBaseUrl(baseUrl: string): void {
    this.upsertSetting(OLLAMA_BASE_URL_KEY, baseUrl);
  }

  getProviderSettings(): ProviderSettingsDTO {
    return {
      defaultProvider: (this.getSetting(DEFAULT_PROVIDER) as ProviderType | null) ?? "ollama",
      ollama: {
        baseUrl: this.getSetting(OLLAMA_BASE_URL_KEY) ?? "http://127.0.0.1:11434"
      },
      openai: {
        apiKey: this.getSetting(OPENAI_API_KEY) ?? "",
        baseUrl: this.getSetting(OPENAI_BASE_URL) ?? "https://api.openai.com/v1"
      },
      google: {
        apiKey: this.getSetting(GOOGLE_API_KEY) ?? "",
        baseUrl:
          this.getSetting(GOOGLE_BASE_URL) ??
          "https://generativelanguage.googleapis.com/v1beta"
      }
    };
  }

  saveProviderSettings(payload: ProviderSettingsDTO): void {
    this.upsertSetting(DEFAULT_PROVIDER, payload.defaultProvider);
    this.upsertSetting(OLLAMA_BASE_URL_KEY, payload.ollama.baseUrl);
    this.upsertSetting(OPENAI_API_KEY, payload.openai.apiKey);
    this.upsertSetting(OPENAI_BASE_URL, payload.openai.baseUrl);
    this.upsertSetting(GOOGLE_API_KEY, payload.google.apiKey);
    this.upsertSetting(GOOGLE_BASE_URL, payload.google.baseUrl);
  }
}
