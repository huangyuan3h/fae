import type { LanguageModelV3 } from "@ai-sdk/provider";
import { createOpenAI } from "@ai-sdk/openai";
import { createGoogleGenerativeAI } from "@ai-sdk/google";
import { createOllama } from "ollama-ai-provider-v2";
import { eq } from "drizzle-orm";
import type { DB } from "../db/types";
import { settings } from "../db/schema";

export type ProviderType = "ollama" | "openai" | "google";

interface ProviderSettings {
  providerConfigs: Array<{
    id: string;
    name: string;
    type: ProviderType;
    apiKey: string;
    baseUrl: string;
    modelId: string;
    enabled: boolean;
  }>;
  defaultProvider: ProviderType;
}

function normalizeOllamaBaseURL(baseURL: string): string {
  const trimmed = baseURL.trim().replace(/\/+$/, "");
  return trimmed.endsWith("/api") ? trimmed : `${trimmed}/api`;
}

function readSetting(db: DB, key: string): string | null {
  const row = db.orm
    .select({ value: settings.value })
    .from(settings)
    .where(eq(settings.key, key))
    .get();
  return row?.value ?? null;
}

export function getProviderSettings(db: DB): ProviderSettings {
  const rawProviderConfigs = readSetting(db, "provider.configs");
  const defaultProvider =
    (readSetting(db, "provider.default") as ProviderType | null) ?? "ollama";

  if (rawProviderConfigs) {
    try {
      const parsed = JSON.parse(rawProviderConfigs) as unknown;
      if (Array.isArray(parsed)) {
        const providerConfigs = parsed
          .map((entry) => normalizeProviderConfig(entry))
          .filter(
            (
              entry
            ): entry is {
              id: string;
              name: string;
              type: ProviderType;
              apiKey: string;
              baseUrl: string;
              modelId: string;
              enabled: boolean;
            } => entry !== null
          );

        return {
          providerConfigs,
          defaultProvider: providerConfigs[0]?.type ?? defaultProvider
        };
      }
    } catch {
      // fall through to legacy fields
    }
  }

  const ollamaBaseUrl =
    readSetting(db, "provider.ollama.baseUrl") ?? "http://127.0.0.1:11434";
  const openaiApiKey = readSetting(db, "provider.openai.apiKey") ?? "";
  const openaiBaseUrl = readSetting(db, "provider.openai.baseUrl") ?? "https://api.openai.com/v1";
  const googleApiKey = readSetting(db, "provider.google.apiKey") ?? "";
  const googleBaseUrl =
    readSetting(db, "provider.google.baseUrl") ??
    "https://generativelanguage.googleapis.com/v1beta";

  const providerConfigs = [
    {
      id: "legacy-ollama",
      name: "Ollama Local",
      type: "ollama" as const,
      apiKey: "",
      baseUrl: ollamaBaseUrl,
      modelId: "",
      enabled: true
    },
    {
      id: "legacy-openai",
      name: "OpenAI Default",
      type: "openai" as const,
      apiKey: openaiApiKey,
      baseUrl: openaiBaseUrl,
      modelId: "",
      enabled: true
    },
    {
      id: "legacy-google",
      name: "Google Default",
      type: "google" as const,
      apiKey: googleApiKey,
      baseUrl: googleBaseUrl,
      modelId: "",
      enabled: true
    }
  ];

  return {
    providerConfigs,
    defaultProvider
  };
}

export function resolveProvider(
  preferred: ProviderType | null | undefined,
  settings: ProviderSettings,
  providerConfigId?: string | null
): ProviderType {
  if (providerConfigId) {
    const matched = settings.providerConfigs.find((config) => config.id === providerConfigId);
    if (matched) {
      return matched.type;
    }
  }
  return preferred ?? settings.defaultProvider;
}

export function getModelForProvider(params: {
  provider: ProviderType;
  modelId: string;
  providerConfigId?: string | null;
  settings: ProviderSettings;
}): LanguageModelV3 {
  const config = selectProviderConfig(params.settings, params.provider, params.providerConfigId);
  const resolvedModelId = params.modelId.trim() || config?.modelId || defaultModelForProvider(params.provider);

  if (params.provider === "ollama") {
    const ollama = createOllama({
      baseURL: normalizeOllamaBaseURL(config?.baseUrl ?? "http://127.0.0.1:11434")
    });
    return ollama.chat(resolvedModelId);
  }

  if (params.provider === "openai") {
    const provider = createOpenAI({
      apiKey: config?.apiKey ?? "",
      baseURL: config?.baseUrl ?? "https://api.openai.com/v1"
    });
    return provider.chat(resolvedModelId);
  }

  const provider = createGoogleGenerativeAI({
    apiKey: config?.apiKey ?? "",
    baseURL: config?.baseUrl ?? "https://generativelanguage.googleapis.com/v1beta"
  });
  return provider(resolvedModelId);
}

function normalizeProviderConfig(input: unknown): {
  id: string;
  name: string;
  type: ProviderType;
  apiKey: string;
  baseUrl: string;
  modelId: string;
  enabled: boolean;
} | null {
  if (!input || typeof input !== "object") {
    return null;
  }

  const item = input as Record<string, unknown>;
  const type = item.type;
  if (type !== "ollama" && type !== "openai" && type !== "google") {
    return null;
  }

  const id = typeof item.id === "string" && item.id.trim().length > 0 ? item.id : "";
  const name = typeof item.name === "string" && item.name.trim().length > 0 ? item.name : "";
  const apiKey = typeof item.apiKey === "string" ? item.apiKey : "";
  const baseUrl = typeof item.baseUrl === "string" && item.baseUrl.trim().length > 0 ? item.baseUrl : "";
  const modelId = typeof item.modelId === "string" ? item.modelId.trim() : "";
  const enabled = typeof item.enabled === "boolean" ? item.enabled : true;

  if (!id || !name || !baseUrl) {
    return null;
  }

  return { id, name, type, apiKey, baseUrl, modelId, enabled };
}

function selectProviderConfig(
  settings: ProviderSettings,
  type: ProviderType,
  providerConfigId?: string | null
) {
  if (providerConfigId) {
    const exact = settings.providerConfigs.find(
      (config) => config.id === providerConfigId && config.type === type
    );
    if (exact) {
      return exact;
    }
  }

  return (
    settings.providerConfigs.find((config) => config.type === type && config.enabled) ??
    settings.providerConfigs.find((config) => config.type === type) ??
    null
  );
}

function defaultModelForProvider(type: ProviderType): string {
  if (type === "openai") {
    return "gpt-4o-mini";
  }
  if (type === "google") {
    return "gemini-2.5-flash";
  }
  return "qwen3:8b";
}
