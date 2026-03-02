import { Hono } from "hono";
import { z } from "zod";
import type { AppBindings } from "../types";
import { ChatController } from "../controllers/chat-controller";

const controller = new ChatController();

const chatSchema = z.object({
  agentId: z.string().min(1),
  message: z.string().min(1)
});

export const chatRoutes = new Hono<AppBindings>();

chatRoutes.post("/", async (c) => {
  const payload = chatSchema.parse(await c.req.json());
  return controller.stream(c, payload);
});
