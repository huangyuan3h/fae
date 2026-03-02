import { randomUUID } from "node:crypto";
import { Hono } from "hono";
import { z } from "zod";
import type { AppBindings } from "../types";

const chatSchema = z.object({
  agentId: z.string().min(1),
  message: z.string().min(1)
});

export const chatRoutes = new Hono<AppBindings>();

function normalizeOllamaBaseURL(baseURL: string): string {
  const trimmed = baseURL.trim().replace(/\/+$/, "");
  return trimmed.endsWith("/api") ? trimmed : `${trimmed}/api`;
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

  const ollamaBaseUrlSetting = db
    .prepare<{ value: string }>("SELECT value FROM settings WHERE key = ?")
    .get("ollama.baseUrl");
  const ollamaBaseURL = normalizeOllamaBaseURL(
    ollamaBaseUrlSetting?.value ?? "http://127.0.0.1:11434"
  );

  const encoder = new TextEncoder();
  let assistantContent = "";

  const stream = new ReadableStream<Uint8Array>({
    async start(controller) {
      try {
        const ollamaResponse = await fetch(`${ollamaBaseURL}/chat`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            model: agent.model,
            stream: true,
            messages: [
              ...(agent.system_prompt
                ? [{ role: "system", content: agent.system_prompt }]
                : []),
              { role: "user", content: payload.message }
            ]
          })
        });

        if (!ollamaResponse.ok || !ollamaResponse.body) {
          throw new Error(
            `Ollama request failed with status ${ollamaResponse.status}`
          );
        }

        const reader = ollamaResponse.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
          const { done, value } = await reader.read();
          if (done) {
            break;
          }

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split("\n");
          buffer = lines.pop() ?? "";

          for (const line of lines) {
            const trimmed = line.trim();
            if (!trimmed) {
              continue;
            }

            let parsed:
              | { message?: { content?: string }; done?: boolean; error?: string }
              | undefined;
            try {
              parsed = JSON.parse(trimmed) as {
                message?: { content?: string };
                done?: boolean;
                error?: string;
              };
            } catch {
              continue;
            }

            if (parsed.error) {
              throw new Error(parsed.error);
            }

            const chunk = parsed.message?.content ?? "";
            if (chunk.length > 0) {
              assistantContent += chunk;
              const event = `data: ${JSON.stringify({ type: "chunk", content: chunk })}\n\n`;
              controller.enqueue(encoder.encode(event));
            }
          }
        }

        db.prepare(
          "INSERT INTO messages (id, agent_id, role, content) VALUES (?, ?, ?, ?)"
        ).run(randomUUID(), payload.agentId, "assistant", assistantContent);

        controller.enqueue(encoder.encode("data: [DONE]\n\n"));
        controller.close();
      } catch (error) {
        logger.error({ error }, "Chat streaming failed");
        const event = `data: ${JSON.stringify({
          type: "error",
          message:
            error instanceof Error ? error.message : "Chat streaming failed"
        })}\n\n`;
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
