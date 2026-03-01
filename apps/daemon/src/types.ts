import type { Logger } from "pino";
import type { DB } from "./db/types";

export interface SkillDefinition {
  id: string;
  name: string;
  execute: (params: unknown) => Promise<unknown> | unknown;
}

export interface AppVariables {
  db: DB;
  logger: Logger;
  startupToken: string;
  skills: SkillDefinition[];
}

export type AppBindings = {
  Variables: AppVariables;
};
