import { createOllama } from "ollama-ai-provider-v2";
import type { LanguageModelV3 } from "@ai-sdk/provider";

function normalizeOllamaBaseURL(baseURL: string): string {
  const trimmed = baseURL.trim().replace(/\/+$/, "");
  if (trimmed.endsWith("/api")) {
    return trimmed;
  }
  return `${trimmed}/api`;
}

export function getModel(modelId: string, baseURL?: string): LanguageModelV3 {
  const ollama = createOllama({
    baseURL: normalizeOllamaBaseURL(
      baseURL ?? process.env.OLLAMA_BASE_URL ?? "http://127.0.0.1:11434"
    )
  });
  return ollama.chat(modelId);
}
