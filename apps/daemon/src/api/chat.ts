import { randomUUID } from "node:crypto";
import { streamText, tool, type ToolSet } from "ai";
import { Hono } from "hono";
import { z } from "zod";
import { getModel } from "../core/llm";
import type { AppBindings } from "../types";

const chatSchema = z.object({
  agentId: z.string().min(1),
  message: z.string().min(1)
});

export const chatRoutes = new Hono<AppBindings>();

function errorToMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function createSkillTools(
  skills: AppBindings["Variables"]["skills"],
  enabledSkillIds: Set<string>
): ToolSet {
  const tools: ToolSet = {};

  for (const skill of skills) {
    if (!enabledSkillIds.has(skill.id)) {
      continue;
    }

    tools[skill.id] = tool({
      description: `Execute skill "${skill.name}" for local assistant tasks.`,
      inputSchema: z.object({}).passthrough(),
      execute: async (input) => {
        return await skill.execute(input);
      }
    });
  }

  return tools;
}

chatRoutes.post("/", async (c) => {
  const payload = chatSchema.parse(await c.req.json());
  const db = c.get("db");
  const logger = c.get("logger");

  const agent = db
    .prepare("SELECT id, model, system_prompt FROM agents WHERE id = ?")
    .get(payload.agentId) as
    | { id: string; model: string; system_prompt: string | null }
    | undefined;

  if (!agent) {
    return c.json(
      {
        ok: false,
        error: {
          code: "AGENT_NOT_FOUND",
          message: "Agent does not exist"
        }
      },
      404
    );
  }

  db.prepare(
    "INSERT INTO messages (id, agent_id, role, content) VALUES (?, ?, ?, ?)"
  ).run(randomUUID(), payload.agentId, "user", payload.message);

  const recentMessages = db
    .prepare<{ role: "user" | "assistant"; content: string }>(
      "SELECT role, content FROM messages WHERE agent_id = ? ORDER BY created_at DESC LIMIT 24"
    )
    .all(payload.agentId)
    .reverse();

  const ollamaBaseUrlSetting = db
    .prepare<{ value: string }>("SELECT value FROM settings WHERE key = ?")
    .get("ollama.baseUrl");
  const ollamaBaseURL = ollamaBaseUrlSetting?.value ?? "http://127.0.0.1:11434";

  const enabledSkillRows = db
    .prepare<{ id: string }>("SELECT id FROM skills WHERE enabled = 1")
    .all();
  const enabledSkillIds = new Set(enabledSkillRows.map((row) => row.id));
  const tools = createSkillTools(c.get("skills"), enabledSkillIds);
  const hasTools = Object.keys(tools).length > 0;

  const encoder = new TextEncoder();

  const stream = new ReadableStream<Uint8Array>({
    async start(controller) {
      const sendEvent = (payload: unknown) => {
        const event = `data: ${JSON.stringify(payload)}\n\n`;
        controller.enqueue(encoder.encode(event));
      };

      let assistantContent = "";
      try {
        const result = streamText({
          model: getModel(
            agent.model,
            ollamaBaseURL
          ) as unknown as Parameters<typeof streamText>[0]["model"],
          system: agent.system_prompt ?? undefined,
          messages: recentMessages,
          tools: hasTools ? tools : undefined,
          onError: ({ error }) => {
            logger.error({ error }, "streamText emitted an error chunk");
          }
        });

        for await (const part of result.fullStream) {
          if (part.type === "text-delta") {
            assistantContent += part.text;
            sendEvent({ type: "chunk", content: part.text });
          } else if (part.type === "reasoning-start") {
            sendEvent({ type: "think-start", id: part.id });
          } else if (part.type === "reasoning-delta") {
            sendEvent({ type: "think", id: part.id, content: part.text });
          } else if (part.type === "reasoning-end") {
            sendEvent({ type: "think-end", id: part.id });
          } else if (part.type === "tool-input-start") {
            sendEvent({
              type: "tool-input-start",
              toolCallId: part.id,
              toolName: part.toolName
            });
          } else if (part.type === "tool-input-delta") {
            sendEvent({
              type: "tool-input-delta",
              toolCallId: part.id,
              delta: part.delta
            });
          } else if (part.type === "tool-input-end") {
            sendEvent({
              type: "tool-input-end",
              toolCallId: part.id
            });
          } else if (part.type === "tool-call") {
            sendEvent({
              type: "tool-call",
              toolCallId: part.toolCallId,
              toolName: part.toolName,
              input: part.input
            });
          } else if (part.type === "tool-result") {
            sendEvent({
              type: "tool-result",
              toolCallId: part.toolCallId,
              toolName: part.toolName,
              output: part.output
            });
          } else if (part.type === "tool-error") {
            sendEvent({
              type: "tool-error",
              toolCallId: part.toolCallId,
              toolName: part.toolName,
              message: errorToMessage(part.error)
            });
          } else if (part.type === "error") {
            sendEvent({ type: "error", message: errorToMessage(part.error) });
          }
        }

        if (assistantContent.trim().length > 0) {
          db.prepare(
            "INSERT INTO messages (id, agent_id, role, content) VALUES (?, ?, ?, ?)"
          ).run(randomUUID(), payload.agentId, "assistant", assistantContent);
        }

        sendEvent({
          type: "final",
          assistantText: assistantContent
        });
        controller.enqueue(encoder.encode("data: [DONE]\n\n"));
      } catch (error) {
        logger.error({ error }, "Chat streaming failed");
        sendEvent({
          type: "error",
          message: errorToMessage(error)
        });
      } finally {
        controller.close();
      }
    }
  });

  return new Response(stream, {
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive"
    }
  });
});
