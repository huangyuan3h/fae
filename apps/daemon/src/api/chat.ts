import { randomUUID } from "node:crypto";
import { Hono } from "hono";
import { streamText } from "ai";
import { z } from "zod";
import { getModel } from "../core/llm";
import type { AppBindings } from "../types";

const chatSchema = z.object({
  agentId: z.string().min(1),
  message: z.string().min(1)
});

export const chatRoutes = new Hono<AppBindings>();

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

  const llmResult = streamText({
    // NOTE: ollama-ai-provider currently exposes a model type that lags latest ai SDK typings.
    model: getModel(agent.model) as never,
    system: agent.system_prompt ?? "You are a helpful local AI assistant.",
    messages: [{ role: "user", content: payload.message }]
  });

  const encoder = new TextEncoder();
  let assistantContent = "";

  const stream = new ReadableStream<Uint8Array>({
    async start(controller) {
      try {
        for await (const chunk of llmResult.textStream) {
          assistantContent += chunk;
          const event = `data: ${JSON.stringify({ type: "chunk", content: chunk })}\n\n`;
          controller.enqueue(encoder.encode(event));
        }

        db.prepare(
          "INSERT INTO messages (id, agent_id, role, content) VALUES (?, ?, ?, ?)"
        ).run(randomUUID(), payload.agentId, "assistant", assistantContent);

        controller.enqueue(encoder.encode("data: [DONE]\n\n"));
        controller.close();
      } catch (error) {
        logger.error({ error }, "Chat streaming failed");
        const event = `data: ${JSON.stringify({ type: "error", message: "Chat streaming failed" })}\n\n`;
        controller.enqueue(encoder.encode(event));
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
