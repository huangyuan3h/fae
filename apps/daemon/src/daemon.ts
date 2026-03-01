import { chmodSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { createLogger } from "./utils/logger";
import { generateStartupToken, cleanupExpiredSessions } from "./utils/auth";
import { initDatabase } from "./db";
import { createApiRouter } from "./api";
import { loadCoreSkills } from "./core/skill-loader";
import type { AppBindings, SkillDefinition } from "./types";

function ensureDir(dirPath: string): void {
  if (!existsSync(dirPath)) {
    mkdirSync(dirPath, { recursive: true });
  }
}

function getProjectRoot(): string {
  return path.resolve(import.meta.dir, "../../..");
}

async function bootstrap(): Promise<void> {
  const dataRoot = path.join(os.homedir(), ".fae");
  const logsDir = path.join(dataRoot, "logs");
  const userSkillsDir = path.join(dataRoot, "skills");
  const runtimeDataDir = path.join(dataRoot, "data");

  ensureDir(dataRoot);
  ensureDir(logsDir);
  ensureDir(userSkillsDir);
  ensureDir(runtimeDataDir);

  const logger = createLogger(path.join(logsDir, "daemon.log"));
  const startupToken = generateStartupToken();

  const startupTokenPath = path.join(dataRoot, "startup-token");
  writeFileSync(startupTokenPath, `${startupToken}\n`, { encoding: "utf8" });
  chmodSync(startupTokenPath, 0o600);

  logger.info(
    { startupTokenPath },
    "Startup token generated and stored in local file"
  );

  const dbPath = path.join(dataRoot, "fae.db");
  const schemaPath = path.join(import.meta.dir, "db", "schema.sql");
  const db = initDatabase({ dbPath, schemaPath, logger });
  cleanupExpiredSessions(db);

  const coreSkillsDir = path.join(getProjectRoot(), "skills", "core");
  const skills: SkillDefinition[] = await loadCoreSkills({
    skillsDir: coreSkillsDir,
    db,
    logger
  });

  const app = new Hono<AppBindings>();
  app.use(
    "*",
    cors({
      origin: "http://localhost:3000",
      allowHeaders: ["Content-Type", "Authorization"],
      allowMethods: ["GET", "POST", "OPTIONS"]
    })
  );

  app.use("*", async (c, next) => {
    c.set("db", db);
    c.set("logger", logger);
    c.set("startupToken", startupToken);
    c.set("skills", skills);
    await next();
  });

  app.route("/api", createApiRouter());

  const hostname = "127.0.0.1";
  const port = 8080;
  const server = Bun.serve({
    hostname,
    port,
    fetch: app.fetch
  });

  logger.info({ hostname, port }, "fae daemon started");

  const shutdown = () => {
    logger.info("Shutting down fae daemon");
    server.stop(true);
    db.close();
    process.exit(0);
  };

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

bootstrap().catch((error) => {
  // Only used during bootstrap failures before logger is initialized.
  console.error("Failed to start daemon", error);
  process.exit(1);
});
