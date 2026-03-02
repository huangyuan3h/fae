import type { LanguageModelV3 } from "@ai-sdk/provider";
import { createOpenAI } from "@ai-sdk/openai";
import { createGoogleGenerativeAI } from "@ai-sdk/google";
import { createOllama } from "ollama-ai-provider-v2";
import { eq } from "drizzle-orm";
import type { DB } from "../db/types";
import { settings } from "../db/schema";

export type ProviderType = "ollama" | "openai" | "google";

interface ProviderSettings {
  defaultProvider: ProviderType;
  ollamaBaseUrl: string;
  openaiApiKey: string;
  openaiBaseUrl: string;
  googleApiKey: string;
  googleBaseUrl: string;
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
  return {
    defaultProvider:
      (readSetting(db, "provider.default") as ProviderType | null) ?? "ollama",
    ollamaBaseUrl:
      readSetting(db, "provider.ollama.baseUrl") ?? "http://127.0.0.1:11434",
    openaiApiKey: readSetting(db, "provider.openai.apiKey") ?? "",
    openaiBaseUrl: readSetting(db, "provider.openai.baseUrl") ?? "https://api.openai.com/v1",
    googleApiKey: readSetting(db, "provider.google.apiKey") ?? "",
    googleBaseUrl:
      readSetting(db, "provider.google.baseUrl") ??
      "https://generativelanguage.googleapis.com/v1beta"
  };
}

export function resolveProvider(
  preferred: ProviderType | null | undefined,
  settings: ProviderSettings
): ProviderType {
  return preferred ?? settings.defaultProvider;
}

export function getModelForProvider(params: {
  provider: ProviderType;
  modelId: string;
  settings: ProviderSettings;
}): LanguageModelV3 {
  if (params.provider === "ollama") {
    const ollama = createOllama({
      baseURL: normalizeOllamaBaseURL(params.settings.ollamaBaseUrl)
    });
    return ollama.chat(params.modelId);
  }

  if (params.provider === "openai") {
    const provider = createOpenAI({
      apiKey: params.settings.openaiApiKey,
      baseURL: params.settings.openaiBaseUrl
    });
    return provider.chat(params.modelId);
  }

  const provider = createGoogleGenerativeAI({
    apiKey: params.settings.googleApiKey,
    baseURL: params.settings.googleBaseUrl
  });
  return provider(params.modelId);
}
