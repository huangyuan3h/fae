import type { Database as BunSQLiteDatabase } from "bun:sqlite";
import type { BunSQLiteDatabase as DrizzleDatabase } from "drizzle-orm/bun-sqlite";
import type * as schema from "./schema";

export interface DB {
  client: BunSQLiteDatabase;
  orm: DrizzleDatabase<typeof schema>;
  close: () => void;
}
