import { streamText, type TextStreamPart, type ToolSet } from "ai";
import type { DB } from "../db/types";
import type { Logger } from "pino";
import {
  getModelForProvider,
  getProviderSettings,
  resolveProvider
} from "../core/llm";
import { AgentRepository } from "../repositories/agent-repository";
import { createSkillTools, parseSkillIds } from "./skill-tool-service";
import type { SkillDefinition } from "../types";

function errorToMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function emitStreamPart(part: TextStreamPart<ToolSet>, emit: (payload: unknown) => void): string {
  switch (part.type) {
    case "text-delta":
      emit({ type: "chunk", content: part.text });
      return part.text;
    case "reasoning-start":
      emit({ type: "think-start", id: part.id });
      return "";
    case "reasoning-delta":
      emit({ type: "think", id: part.id, content: part.text });
      return "";
    case "reasoning-end":
      emit({ type: "think-end", id: part.id });
      return "";
    case "tool-input-start":
      emit({ type: "tool-input-start", toolCallId: part.id, toolName: part.toolName });
      return "";
    case "tool-input-delta":
      emit({ type: "tool-input-delta", toolCallId: part.id, delta: part.delta });
      return "";
    case "tool-input-end":
      emit({ type: "tool-input-end", toolCallId: part.id });
      return "";
    case "tool-call":
      emit({
        type: "tool-call",
        toolCallId: part.toolCallId,
        toolName: part.toolName,
        input: part.input
      });
      return "";
    case "tool-result":
      emit({
        type: "tool-result",
        toolCallId: part.toolCallId,
        toolName: part.toolName,
        output: part.output
      });
      return "";
    case "tool-error":
      emit({
        type: "tool-error",
        toolCallId: part.toolCallId,
        toolName: part.toolName,
        message: errorToMessage(part.error)
      });
      return "";
    case "error":
      emit({ type: "error", message: errorToMessage(part.error) });
      return "";
    default:
      return "";
  }
}

export function buildAgentChatStream(params: {
  db: DB;
  logger: Logger;
  skills: SkillDefinition[];
  agentId: string;
  message: string;
}): { notFound: boolean; stream?: ReadableStream<Uint8Array> } {
  const agentRepo = new AgentRepository(params.db);
  const agent = agentRepo.findForChat(params.agentId);

  if (!agent) {
    return { notFound: true };
  }

  agentRepo.insertUserMessage(params.agentId, params.message);

  const providerSettings = getProviderSettings(params.db);
  const provider = resolveProvider(
    agent.provider,
    providerSettings,
    agent.provider_config_id
  );
  const requestedSkillIds = parseSkillIds(agent.skills_json);
  const enabledSkillIds = agentRepo.enabledSkillIds();
  const tools = createSkillTools({
    allSkills: params.skills,
    enabledSkillIds,
    requestedSkillIds
  });

  const recentMessages = agentRepo.recentMessages(params.agentId);
  const encoder = new TextEncoder();

  const stream = new ReadableStream<Uint8Array>({
    start: async (controller) => {
      const emit = (payload: unknown) => {
        controller.enqueue(encoder.encode(`data: ${JSON.stringify(payload)}\n\n`));
      };

      let assistantContent = "";

      try {
        const result = streamText({
          model: getModelForProvider({
            provider,
            modelId: agent.model,
            providerConfigId: agent.provider_config_id,
            settings: providerSettings
          }),
          system: agent.system_prompt ?? undefined,
          messages: recentMessages,
          tools: Object.keys(tools).length > 0 ? tools : undefined,
          providerOptions: {
            ollama: { think: true }
          },
          onError: ({ error }) => {
            params.logger.error({ error }, "streamText emitted an error chunk");
          }
        });

        for await (const part of result.fullStream) {
          assistantContent += emitStreamPart(part, emit);
        }

        if (assistantContent.trim().length > 0) {
          agentRepo.insertAssistantMessage(params.agentId, assistantContent);
        }

        emit({ type: "final", assistantText: assistantContent });
        controller.enqueue(encoder.encode("data: [DONE]\n\n"));
      } catch (error) {
        params.logger.error({ error }, "Chat streaming failed");
        emit({ type: "error", message: errorToMessage(error) });
      } finally {
        controller.close();
      }
    }
  });

  return { notFound: false, stream };
}
