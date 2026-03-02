import { randomBytes } from "node:crypto";
import type { MiddlewareHandler } from "hono";
import { HTTPException } from "hono/http-exception";
import { and, eq, gt, lte } from "drizzle-orm";
import type { AppBindings } from "../types";
import type { DB } from "../db/types";
import { sessions } from "../db/schema";

const DEFAULT_SESSION_TTL_SECONDS = 60 * 60 * 24 * 7;

function randomToken(size = 32): string {
  return randomBytes(size).toString("hex");
}

export function generateStartupToken(): string {
  return randomToken(24);
}

export function createSessionToken(
  db: DB,
  ttlSeconds = DEFAULT_SESSION_TTL_SECONDS
): string {
  const token = randomToken(32);
  const now = Math.floor(Date.now() / 1000);
  const expiresAt = now + ttlSeconds;
  db.orm.insert(sessions).values({
    token,
    createdAt: now,
    expiresAt
  }).run();
  return token;
}

export function validateSessionToken(
  db: DB,
  token: string
): boolean {
  const now = Math.floor(Date.now() / 1000);
  const row = db.orm
    .select({ token: sessions.token })
    .from(sessions)
    .where(and(eq(sessions.token, token), gt(sessions.expiresAt, now)))
    .get();
  return Boolean(row);
}

export function cleanupExpiredSessions(db: DB): void {
  const now = Math.floor(Date.now() / 1000);
  db.orm.delete(sessions).where(lte(sessions.expiresAt, now)).run();
}

export function requireAuth(): MiddlewareHandler<AppBindings> {
  return async (c, next) => {
    const authorization = c.req.header("authorization");
    if (!authorization?.startsWith("Bearer ")) {
      throw new HTTPException(401, { message: "Missing or invalid authorization header" });
    }

    const token = authorization.slice("Bearer ".length).trim();
    if (token.length === 0) {
      throw new HTTPException(401, { message: "Empty session token" });
    }

    const db = c.get("db");
    if (!validateSessionToken(db, token)) {
      throw new HTTPException(401, { message: "Session token is invalid or expired" });
    }

    await next();
  };
}
