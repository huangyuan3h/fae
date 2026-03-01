import { createOllama } from "ollama-ai-provider";

const ollama = createOllama({
  baseURL: process.env.OLLAMA_BASE_URL ?? "http://127.0.0.1:11434/api"
});

export function getModel(modelId: string) {
  return ollama(modelId);
}
