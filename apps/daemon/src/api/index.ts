import { Hono } from "hono";
import { HTTPException } from "hono/http-exception";
import { ZodError } from "zod";
import { authRoutes } from "./auth";
import { agentRoutes } from "./agents";
import { chatRoutes } from "./chat";
import { skillRoutes } from "./skills";
import { requireAuth } from "../utils/auth";
import type { AppBindings } from "../types";

export function createApiRouter(): Hono<AppBindings> {
  const api = new Hono<AppBindings>();

  api.get("/health", (c) => c.json({ ok: true, data: { status: "ok" } }));
  api.route("/auth", authRoutes);

  const protectedApi = new Hono<AppBindings>();
  protectedApi.use("*", requireAuth());
  protectedApi.route("/agents", agentRoutes);
  protectedApi.route("/chat", chatRoutes);
  protectedApi.route("/skills", skillRoutes);
  api.route("/", protectedApi);

  api.onError((error, c) => {
    if (error instanceof ZodError) {
      return c.json(
        {
          ok: false,
          error: {
            code: "VALIDATION_ERROR",
            message: "Request payload validation failed",
            details: error.issues
          }
        },
        400
      );
    }

    if (error instanceof HTTPException) {
      return c.json(
        {
          ok: false,
          error: {
            code: "HTTP_ERROR",
            message: error.message
          }
        },
        error.status
      );
    }

    c.get("logger").error({ error }, "Unhandled API error");
    return c.json(
      {
        ok: false,
        error: {
          code: "INTERNAL_ERROR",
          message: "Unexpected server error"
        }
      },
      500
    );
  });

  return api;
}
