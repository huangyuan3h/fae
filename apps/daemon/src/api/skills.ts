import { Hono } from "hono";
import type { AppBindings } from "../types";

export const skillRoutes = new Hono<AppBindings>();

skillRoutes.get("/", (c) => {
  const rows = c
    .get("db")
    .prepare("SELECT id, name, enabled FROM skills ORDER BY id ASC")
    .all();

  return c.json({
    ok: true,
    data: rows
  });
});
