import type { Context } from "hono";
import type { AppBindings } from "../types";
import { ChannelsService } from "../services/channels-service";

export class ChannelsController {
  list(c: Context<AppBindings>) {
    const service = new ChannelsService(c.get("db"), c.get("skills"));
    return c.json({ ok: true, data: service.list() });
  }

  get(c: Context<AppBindings>, id: string) {
    const service = new ChannelsService(c.get("db"), c.get("skills"));
    const channel = service.get(id);

    if (!channel) {
      return c.json(
        {
          ok: false,
          error: {
            code: "CHANNEL_NOT_FOUND",
            message: "Channel does not exist"
          }
        },
        404
      );
    }

    return c.json({ ok: true, data: channel });
  }

  create(
    c: Context<AppBindings>,
    payload: { name: string; topic: string; users: string[]; agentIds: string[] }
  ) {
    const service = new ChannelsService(c.get("db"), c.get("skills"));
    const created = service.create(payload);
    return c.json({ ok: true, data: created }, 201);
  }

  update(
    c: Context<AppBindings>,
    id: string,
    payload: { name: string; topic: string; users: string[]; agentIds: string[] }
  ) {
    const service = new ChannelsService(c.get("db"), c.get("skills"));
    const updated = service.update(id, payload);
    if (!updated) {
      return c.json(
        {
          ok: false,
          error: {
            code: "CHANNEL_NOT_FOUND",
            message: "Channel does not exist"
          }
        },
        404
      );
    }

    return c.json({ ok: true, data: updated });
  }

  delete(c: Context<AppBindings>, id: string) {
    const service = new ChannelsService(c.get("db"), c.get("skills"));
    const ok = service.delete(id);

    if (!ok) {
      return c.json(
        {
          ok: false,
          error: {
            code: "CHANNEL_NOT_FOUND",
            message: "Channel does not exist"
          }
        },
        404
      );
    }

    return c.json({ ok: true, data: { id } });
  }

  async sendMessage(
    c: Context<AppBindings>,
    payload: { channelId: string; message: string; userName: string }
  ) {
    const service = new ChannelsService(c.get("db"), c.get("skills"));
    const result = await service.sendMessage(payload);

    if (!result) {
      return c.json(
        {
          ok: false,
          error: {
            code: "CHANNEL_NOT_FOUND",
            message: "Channel does not exist"
          }
        },
        404
      );
    }

    return c.json({ ok: true, data: result });
  }
}
