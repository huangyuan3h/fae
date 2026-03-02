import { Hono } from "hono";
import { eq } from "drizzle-orm";
import { z } from "zod";
import type { AppBindings } from "../types";
import { skills } from "../db/schema";

export const skillRoutes = new Hono<AppBindings>();

const updateSkillSchema = z.object({
  enabled: z.boolean()
});

skillRoutes.get("/", (c) => {
  const rows = c.get("db").orm.select().from(skills).orderBy(skills.id).all();

  return c.json({
    ok: true,
    data: rows
  });
});

skillRoutes.put("/:id", async (c) => {
  const id = c.req.param("id");
  const payload = updateSkillSchema.parse(await c.req.json());
  const enabled = payload.enabled ? 1 : 0;

  c.get("db").orm
    .update(skills)
    .set({ enabled })
    .where(eq(skills.id, id))
    .run();

  const updated = c.get("db").orm
    .select()
    .from(skills)
    .where(eq(skills.id, id))
    .get();

  if (!updated) {
    return c.json(
      {
        ok: false,
        error: {
          code: "NOT_FOUND",
          message: "Skill not found"
        }
      },
      404
    );
  }

  return c.json({
    ok: true,
    data: updated
  });
});
