import { streamText } from "ai";
import type { DB } from "../db/types";
import type { SkillDefinition } from "../types";
import {
  getModelForProvider,
  getProviderSettings,
  resolveProvider
} from "../core/llm";
import { ChannelRepository } from "../repositories/channel-repository";
import { createSkillTools, parseSkillIds } from "./skill-tool-service";

export class ChannelsService {
  private readonly repo: ChannelRepository;
  private readonly db: DB;

  constructor(
    db: DB,
    private readonly skills: SkillDefinition[]
  ) {
    this.db = db;
    this.repo = new ChannelRepository(db);
  }

  list() {
    return this.repo.listChannels();
  }

  get(id: string) {
    return this.repo.loadChannel(id);
  }

  create(payload: {
    name: string;
    topic: string;
    users: string[];
    agentIds: string[];
  }) {
    const id = this.repo.createChannel(payload);
    return this.repo.loadChannel(id);
  }

  update(
    id: string,
    payload: { name: string; topic: string; users: string[]; agentIds: string[] }
  ) {
    const ok = this.repo.updateChannel(id, payload);
    if (!ok) {
      return null;
    }
    return this.repo.loadChannel(id);
  }

  delete(id: string): boolean {
    return this.repo.deleteChannel(id);
  }

  async sendMessage(payload: {
    channelId: string;
    message: string;
    userName: string;
  }) {
    const channel = this.repo.loadChannel(payload.channelId);
    if (!channel) {
      return null;
    }

    this.repo.insertUserMessage(payload.channelId, payload.userName, payload.message);

    const members = this.repo.membersForChat(payload.channelId);
    const providerSettings = getProviderSettings(this.db);
    const enabledSkillIds = this.repo.enabledSkillIds();

    const replies: Array<{ senderId: string; senderName: string; content: string }> = [];

    for (const member of members) {
      const tools = createSkillTools({
        allSkills: this.skills,
        enabledSkillIds,
        requestedSkillIds: parseSkillIds(member.skills_json)
      });

      const result = streamText({
        model: getModelForProvider({
          provider: resolveProvider(
            member.provider,
            providerSettings,
            member.provider_config_id
          ),
          modelId: member.model,
          providerConfigId: member.provider_config_id,
          settings: providerSettings
        }),
        system: member.system_prompt ?? undefined,
        prompt: payload.message,
        tools: Object.keys(tools).length > 0 ? tools : undefined,
        providerOptions: {
          ollama: { think: true }
        }
      });

      const content = (await result.text).trim();
      if (!content) {
        continue;
      }

      this.repo.insertAgentMessage(payload.channelId, member.id, member.name, content);
      replies.push({ senderId: member.id, senderName: member.name, content });
    }

    return {
      replies,
      channel: this.repo.loadChannel(payload.channelId)
    };
  }
}
