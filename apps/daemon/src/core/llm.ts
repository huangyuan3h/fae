import { createOllama } from "ollama-ai-provider";

function normalizeOllamaBaseURL(baseURL: string): string {
  const trimmed = baseURL.trim().replace(/\/+$/, "");
  if (trimmed.endsWith("/api")) {
    return trimmed;
  }
  return `${trimmed}/api`;
}

export function getModel(modelId: string, baseURL?: string) {
  const ollama = createOllama({
    baseURL: normalizeOllamaBaseURL(
      baseURL ?? process.env.OLLAMA_BASE_URL ?? "http://127.0.0.1:11434"
    )
  });
  return ollama(modelId);
}
