import { sql } from "drizzle-orm";
import { integer, sqliteTable, text } from "drizzle-orm/sqlite-core";

const nowUnix = sql`(strftime('%s', 'now'))`;

export const agents = sqliteTable("agents", {
  id: text("id").primaryKey(),
  name: text("name").notNull(),
  provider: text("provider").notNull().default("ollama"),
  providerConfigId: text("provider_config_id"),
  model: text("model").default("qwen2.5:7b"),
  systemPrompt: text("system_prompt"),
  avatarUrl: text("avatar_url"),
  skillsJson: text("skills_json").notNull().default("[]"),
  createdAt: integer("created_at").default(nowUnix)
});

export const messages = sqliteTable("messages", {
  id: text("id").primaryKey(),
  agentId: text("agent_id").notNull(),
  role: text("role").notNull(),
  content: text("content").notNull(),
  createdAt: integer("created_at").default(nowUnix)
});

export const skills = sqliteTable("skills", {
  id: text("id").primaryKey(),
  name: text("name").notNull(),
  enabled: integer("enabled").default(1)
});

export const sessions = sqliteTable("sessions", {
  token: text("token").primaryKey(),
  createdAt: integer("created_at").default(nowUnix),
  expiresAt: integer("expires_at").notNull()
});

export const settings = sqliteTable("settings", {
  key: text("key").primaryKey(),
  value: text("value").notNull(),
  updatedAt: integer("updated_at").default(nowUnix)
});

export const channels = sqliteTable("channels", {
  id: text("id").primaryKey(),
  name: text("name").notNull(),
  topic: text("topic").notNull().default(""),
  createdAt: integer("created_at").default(nowUnix)
});

export const channelUsers = sqliteTable("channel_users", {
  channelId: text("channel_id").notNull(),
  userName: text("user_name").notNull(),
  createdAt: integer("created_at").default(nowUnix)
});

export const channelMembers = sqliteTable("channel_members", {
  channelId: text("channel_id").notNull(),
  agentId: text("agent_id").notNull(),
  createdAt: integer("created_at").default(nowUnix)
});

export const channelMessages = sqliteTable("channel_messages", {
  id: text("id").primaryKey(),
  channelId: text("channel_id").notNull(),
  senderType: text("sender_type").notNull(),
  senderId: text("sender_id").notNull(),
  senderName: text("sender_name").notNull(),
  content: text("content").notNull(),
  createdAt: integer("created_at").default(nowUnix)
});
