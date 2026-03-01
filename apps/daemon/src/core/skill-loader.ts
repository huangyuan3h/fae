import { pathToFileURL } from "node:url";
import { existsSync, readdirSync } from "node:fs";
import path from "node:path";
import type { Logger } from "pino";
import type { SkillDefinition } from "../types";
import type { DB } from "../db/types";

interface SkillModule {
  id?: string;
  name?: string;
  execute?: (params: unknown) => Promise<unknown> | unknown;
}

function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`Skill execution timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    promise
      .then((value) => {
        clearTimeout(timer);
        resolve(value);
      })
      .catch((error) => {
        clearTimeout(timer);
        reject(error);
      });
  });
}

export async function loadCoreSkills(params: {
  skillsDir: string;
  db: DB;
  logger: Logger;
  timeoutMs?: number;
}): Promise<SkillDefinition[]> {
  const timeoutMs = params.timeoutMs ?? 30_000;
  const skills: SkillDefinition[] = [];

  if (!existsSync(params.skillsDir)) {
    params.logger.warn({ skillsDir: params.skillsDir }, "Skills directory not found");
    return skills;
  }

  const entries = readdirSync(params.skillsDir, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isFile()) {
      continue;
    }
    const ext = path.extname(entry.name);
    if (ext !== ".ts" && ext !== ".js" && ext !== ".mjs") {
      continue;
    }

    const filePath = path.join(params.skillsDir, entry.name);
    const moduleURL = pathToFileURL(filePath).href;
    let mod: SkillModule;
    try {
      mod = (await import(moduleURL)) as SkillModule;
    } catch (error) {
      params.logger.error({ error, filePath }, "Failed to import skill module");
      continue;
    }

    if (typeof mod.execute !== "function") {
      params.logger.warn({ filePath }, "Skill missing execute(params) export");
      continue;
    }

    const skillId = mod.id ?? path.basename(entry.name, ext);
    const skillName = mod.name ?? skillId;
    const execute = mod.execute;

    const wrapped: SkillDefinition = {
      id: skillId,
      name: skillName,
      execute: (skillParams) =>
        withTimeout(Promise.resolve(execute(skillParams)), timeoutMs)
    };

    params.db
      .prepare(
        "INSERT INTO skills (id, name, enabled) VALUES (?, ?, 1) ON CONFLICT(id) DO UPDATE SET name = excluded.name, enabled = 1"
      )
      .run(wrapped.id, wrapped.name);

    skills.push(wrapped);
  }

  params.logger.info({ count: skills.length }, "Core skills loaded");
  return skills;
}
