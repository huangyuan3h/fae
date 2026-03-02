import { randomUUID } from "node:crypto";
import { asc, desc, eq, sql } from "drizzle-orm";
import type { DB } from "../db/types";
import type { ProviderType } from "../core/llm";
import {
  agents,
  channelMembers,
  channelMessages,
  channels,
  channelUsers,
  skills
} from "../db/schema";

export interface ChannelSummary {
  id: string;
  name: string;
  topic: string;
  created_at: number;
  member_count: number;
  user_count: number;
}

export interface ChannelMessageRow {
  id: string;
  sender_type: string;
  sender_id: string;
  sender_name: string;
  content: string;
  created_at: number;
}

export interface ChannelMemberRow {
  id: string;
  name: string;
  provider: ProviderType | null;
  model: string;
  system_prompt: string | null;
  skills_json: string | null;
}

export interface ChannelDetail {
  id: string;
  name: string;
  topic: string;
  created_at: number;
  users: string[];
  members: Array<{ id: string; name: string }>;
  messages: ChannelMessageRow[];
}

export class ChannelRepository {
  constructor(private readonly db: DB) {}

  listChannels(): ChannelSummary[] {
    const rows = this.db.orm
      .select({
        id: channels.id,
        name: channels.name,
        topic: channels.topic,
        created_at: channels.createdAt,
        member_count: sql<number>`(SELECT COUNT(*) FROM channel_members cm WHERE cm.channel_id = ${channels.id})`,
        user_count: sql<number>`(SELECT COUNT(*) FROM channel_users cu WHERE cu.channel_id = ${channels.id})`
      })
      .from(channels)
      .orderBy(desc(channels.createdAt))
      .all();

    return rows.map((row) => ({
      ...row,
      created_at: row.created_at ?? 0,
      member_count: Number(row.member_count ?? 0),
      user_count: Number(row.user_count ?? 0)
    }));
  }

  loadChannel(channelId: string): ChannelDetail | null {
    const channel = this.db.orm
      .select({
        id: channels.id,
        name: channels.name,
        topic: channels.topic,
        created_at: channels.createdAt
      })
      .from(channels)
      .where(eq(channels.id, channelId))
      .get();

    if (!channel) {
      return null;
    }

    const users = this.db.orm
      .select({ user_name: channelUsers.userName })
      .from(channelUsers)
      .where(eq(channelUsers.channelId, channelId))
      .orderBy(asc(channelUsers.userName))
      .all()
      .map((row) => row.user_name);

    const members = this.db.orm
      .select({ id: agents.id, name: agents.name })
      .from(channelMembers)
      .innerJoin(agents, eq(channelMembers.agentId, agents.id))
      .where(eq(channelMembers.channelId, channelId))
      .orderBy(asc(agents.name))
      .all();

    const messagesRows = this.db.orm
      .select({
        id: channelMessages.id,
        sender_type: channelMessages.senderType,
        sender_id: channelMessages.senderId,
        sender_name: channelMessages.senderName,
        content: channelMessages.content,
        created_at: channelMessages.createdAt
      })
      .from(channelMessages)
      .where(eq(channelMessages.channelId, channelId))
      .orderBy(asc(channelMessages.createdAt))
      .all();

    return {
      id: channel.id,
      name: channel.name,
      topic: channel.topic,
      created_at: channel.created_at ?? 0,
      users,
      members,
      messages: messagesRows.map((row) => ({
        ...row,
        created_at: row.created_at ?? 0
      }))
    };
  }

  createChannel(payload: {
    name: string;
    topic: string;
    users: string[];
    agentIds: string[];
  }): string {
    const id = randomUUID();
    this.db.orm
      .insert(channels)
      .values({ id, name: payload.name, topic: payload.topic })
      .run();

    this.replaceUsers(id, payload.users);
    this.replaceMembers(id, payload.agentIds);
    return id;
  }

  updateChannel(
    id: string,
    payload: { name: string; topic: string; users: string[]; agentIds: string[] }
  ): boolean {
    const existing = this.db.orm
      .select({ id: channels.id })
      .from(channels)
      .where(eq(channels.id, id))
      .get();

    if (!existing) {
      return false;
    }

    this.db.orm
      .update(channels)
      .set({ name: payload.name, topic: payload.topic })
      .where(eq(channels.id, id))
      .run();

    this.replaceUsers(id, payload.users);
    this.replaceMembers(id, payload.agentIds);
    return true;
  }

  deleteChannel(id: string): boolean {
    const existing = this.db.orm
      .select({ id: channels.id })
      .from(channels)
      .where(eq(channels.id, id))
      .get();

    if (!existing) {
      return false;
    }

    this.db.orm.delete(channelMessages).where(eq(channelMessages.channelId, id)).run();
    this.db.orm.delete(channelUsers).where(eq(channelUsers.channelId, id)).run();
    this.db.orm.delete(channelMembers).where(eq(channelMembers.channelId, id)).run();

    this.db.orm.delete(channels).where(eq(channels.id, id)).run();
    return true;
  }

  insertUserMessage(channelId: string, userName: string, content: string): void {
    this.db.orm
      .insert(channelMessages)
      .values({
        id: randomUUID(),
        channelId,
        senderType: "user",
        senderId: userName,
        senderName: userName,
        content
      })
      .run();
  }

  insertAgentMessage(
    channelId: string,
    agentId: string,
    agentName: string,
    content: string
  ): void {
    this.db.orm
      .insert(channelMessages)
      .values({
        id: randomUUID(),
        channelId,
        senderType: "agent",
        senderId: agentId,
        senderName: agentName,
        content
      })
      .run();
  }

  membersForChat(channelId: string): ChannelMemberRow[] {
    const rows = this.db.orm
      .select({
        id: agents.id,
        name: agents.name,
        provider: agents.provider,
        model: agents.model,
        system_prompt: agents.systemPrompt,
        skills_json: agents.skillsJson
      })
      .from(channelMembers)
      .innerJoin(agents, eq(channelMembers.agentId, agents.id))
      .where(eq(channelMembers.channelId, channelId))
      .orderBy(asc(agents.createdAt))
      .all();

    return rows.map((row) => ({
      id: row.id,
      name: row.name,
      provider: row.provider as ProviderType,
      model: row.model ?? "qwen2.5:7b",
      system_prompt: row.system_prompt,
      skills_json: row.skills_json
    }));
  }

  enabledSkillIds(): Set<string> {
    const rows = this.db.orm
      .select({ id: skills.id })
      .from(skills)
      .where(eq(skills.enabled, 1))
      .all();
    return new Set(rows.map((row) => row.id));
  }

  private replaceUsers(channelId: string, usersList: string[]): void {
    this.db.orm.delete(channelUsers).where(eq(channelUsers.channelId, channelId)).run();

    if (usersList.length === 0) {
      return;
    }

    this.db.orm
      .insert(channelUsers)
      .values(usersList.map((userName) => ({ channelId, userName })))
      .run();
  }

  private replaceMembers(channelId: string, agentIds: string[]): void {
    this.db.orm.delete(channelMembers).where(eq(channelMembers.channelId, channelId)).run();

    if (agentIds.length === 0) {
      return;
    }

    this.db.orm
      .insert(channelMembers)
      .values(agentIds.map((agentId) => ({ channelId, agentId })))
      .run();
  }
}
