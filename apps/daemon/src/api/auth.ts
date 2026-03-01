import { Hono } from "hono";
import { z } from "zod";
import { createSessionToken } from "../utils/auth";
import type { AppBindings } from "../types";

const loginSchema = z.object({
  token: z.string().min(1)
});

export const authRoutes = new Hono<AppBindings>();

authRoutes.post("/login", async (c) => {
  const payload = loginSchema.parse(await c.req.json());
  const startupToken = c.get("startupToken");

  if (payload.token !== startupToken) {
    return c.json(
      {
        ok: false,
        error: {
          code: "INVALID_STARTUP_TOKEN",
          message: "Startup token is invalid"
        }
      },
      401
    );
  }

  const sessionToken = createSessionToken(c.get("db"));
  return c.json({
    ok: true,
    data: {
      sessionToken
    }
  });
});
