import { Hono } from "hono";
import { z } from "zod";
import type { AppBindings } from "../types";
import { ChannelsController } from "../controllers/channels-controller";

const controller = new ChannelsController();

const channelSchema = z.object({
  name: z.string().min(1),
  topic: z.string().default(""),
  users: z.array(z.string().min(1)).default([]),
  agentIds: z.array(z.string().uuid()).default([])
});

const channelMessageSchema = z.object({
  message: z.string().min(1),
  userName: z.string().min(1).default("You")
});

export const channelRoutes = new Hono<AppBindings>();

channelRoutes.get("/", (c) => controller.list(c));

channelRoutes.get("/:id", (c) => controller.get(c, c.req.param("id")));

channelRoutes.post("/", async (c) => {
  const payload = channelSchema.parse(await c.req.json());
  return controller.create(c, payload);
});

channelRoutes.put("/:id", async (c) => {
  const payload = channelSchema.parse(await c.req.json());
  return controller.update(c, c.req.param("id"), payload);
});

channelRoutes.delete("/:id", (c) => controller.delete(c, c.req.param("id")));

channelRoutes.post("/:id/chat", async (c) => {
  const payload = channelMessageSchema.parse(await c.req.json());
  return controller.sendMessage(c, {
    channelId: c.req.param("id"),
    message: payload.message,
    userName: payload.userName
  });
});
