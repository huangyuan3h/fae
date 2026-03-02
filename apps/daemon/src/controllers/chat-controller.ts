import type { Context } from "hono";
import type { AppBindings } from "../types";
import { buildAgentChatStream } from "../services/chat-service";

export class ChatController {
  stream(c: Context<AppBindings>, payload: { agentId: string; message: string }) {
    const result = buildAgentChatStream({
      db: c.get("db"),
      logger: c.get("logger"),
      skills: c.get("skills"),
      agentId: payload.agentId,
      message: payload.message
    });

    if (result.notFound || !result.stream) {
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

    return new Response(result.stream, {
      headers: {
        "Content-Type": "text/event-stream",
        "Cache-Control": "no-cache",
        Connection: "keep-alive"
      }
    });
  }
}
