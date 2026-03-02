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
const PROVIDER_CONFIGS_KEY = "provider.configs";

const defaultBaseUrl: Record<ProviderType, string> = {
  ollama: "http://127.0.0.1:11434",
  openai: "https://api.openai.com/v1",
  google: "https://generativelanguage.googleapis.com/v1beta"
};

export interface ProviderConfigDTO {
  id: string;
  name: string;
  type: ProviderType;
  apiKey: string;
  baseUrl: string;
  enabled: boolean;
}

export interface ProviderSettingsDTO {
  providerConfigs: ProviderConfigDTO[];
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
    const existing = this.getProviderSettings().providerConfigs;
    const next = [...existing];
    const index = next.findIndex((config) => config.type === "ollama");

    if (index >= 0) {
      next[index] = {
        ...next[index],
        baseUrl
      };
    } else {
      next.push({
        id: crypto.randomUUID(),
        name: "Ollama Local",
        type: "ollama",
        apiKey: "",
        baseUrl,
        enabled: true
      });
    }

    this.upsertSetting(PROVIDER_CONFIGS_KEY, JSON.stringify(next));
  }

  getProviderSettings(): ProviderSettingsDTO {
    const configsFromStore = this.getSetting(PROVIDER_CONFIGS_KEY);
    if (configsFromStore) {
      try {
        const parsed = JSON.parse(configsFromStore) as unknown;
        if (Array.isArray(parsed)) {
          const normalized = parsed
            .map((item, index) => this.normalizeProviderConfig(item, index))
            .filter((item): item is ProviderConfigDTO => item !== null);
          return { providerConfigs: normalized };
        }
      } catch {
        // fall back to legacy settings keys below
      }
    }

    const legacy: ProviderConfigDTO[] = [
      {
        id: "legacy-ollama",
        name: "Ollama Local",
        type: "ollama",
        apiKey: "",
        baseUrl: this.getSetting(OLLAMA_BASE_URL_KEY) ?? defaultBaseUrl.ollama,
        enabled: true
      }
    ];

    const openaiApiKey = this.getSetting(OPENAI_API_KEY) ?? "";
    const openaiBaseUrl = this.getSetting(OPENAI_BASE_URL) ?? defaultBaseUrl.openai;
    if (openaiApiKey || openaiBaseUrl !== defaultBaseUrl.openai) {
      legacy.push({
        id: "legacy-openai",
        name: "OpenAI Default",
        type: "openai",
        apiKey: openaiApiKey,
        baseUrl: openaiBaseUrl,
        enabled: true
      });
    }

    const googleApiKey = this.getSetting(GOOGLE_API_KEY) ?? "";
    const googleBaseUrl = this.getSetting(GOOGLE_BASE_URL) ?? defaultBaseUrl.google;
    if (googleApiKey || googleBaseUrl !== defaultBaseUrl.google) {
      legacy.push({
        id: "legacy-google",
        name: "Google Default",
        type: "google",
        apiKey: googleApiKey,
        baseUrl: googleBaseUrl,
        enabled: true
      });
    }

    return { providerConfigs: legacy };
  }

  saveProviderSettings(payload: ProviderSettingsDTO): void {
    const normalized = payload.providerConfigs
      .map((item, index) => this.normalizeProviderConfig(item, index))
      .filter((item): item is ProviderConfigDTO => item !== null);

    this.upsertSetting(PROVIDER_CONFIGS_KEY, JSON.stringify(normalized));

    const firstByType = (type: ProviderType): ProviderConfigDTO | undefined =>
      normalized.find((config) => config.type === type && config.enabled) ??
      normalized.find((config) => config.type === type);

    const firstOllama = firstByType("ollama");
    const firstOpenai = firstByType("openai");
    const firstGoogle = firstByType("google");

    this.upsertSetting(OLLAMA_BASE_URL_KEY, firstOllama?.baseUrl ?? defaultBaseUrl.ollama);
    this.upsertSetting(OPENAI_API_KEY, firstOpenai?.apiKey ?? "");
    this.upsertSetting(OPENAI_BASE_URL, firstOpenai?.baseUrl ?? defaultBaseUrl.openai);
    this.upsertSetting(GOOGLE_API_KEY, firstGoogle?.apiKey ?? "");
    this.upsertSetting(GOOGLE_BASE_URL, firstGoogle?.baseUrl ?? defaultBaseUrl.google);
    this.upsertSetting(DEFAULT_PROVIDER, normalized[0]?.type ?? "ollama");
  }

  private normalizeProviderConfig(
    input: unknown,
    index: number
  ): ProviderConfigDTO | null {
    if (!input || typeof input !== "object") {
      return null;
    }

    const item = input as Record<string, unknown>;
    const type = item.type;
    if (type !== "ollama" && type !== "openai" && type !== "google") {
      return null;
    }

    const idValue = typeof item.id === "string" && item.id.trim().length > 0
      ? item.id.trim()
      : `provider-${type}-${index + 1}`;
    const nameValue = typeof item.name === "string" && item.name.trim().length > 0
      ? item.name.trim()
      : `${type.toUpperCase()} ${index + 1}`;
    const apiKeyValue = typeof item.apiKey === "string" ? item.apiKey : "";
    const baseUrlValue = typeof item.baseUrl === "string" && item.baseUrl.trim().length > 0
      ? item.baseUrl
      : defaultBaseUrl[type];
    const enabledValue = typeof item.enabled === "boolean" ? item.enabled : true;

    return {
      id: idValue,
      name: nameValue,
      type,
      apiKey: apiKeyValue,
      baseUrl: baseUrlValue,
      enabled: enabledValue
    };
  }
}
