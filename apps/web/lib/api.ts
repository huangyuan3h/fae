const daemonPort = process.env.NEXT_PUBLIC_DAEMON_PORT ?? "8787";
export const API_BASE =
  process.env.NEXT_PUBLIC_API_BASE ?? `http://127.0.0.1:${daemonPort}`;

interface ApiErrorShape {
  code?: string;
  message?: string;
}

interface ApiResponse<T> {
  ok: boolean;
  data?: T;
  error?: ApiErrorShape;
}

async function requestJson<T>(
  path: string,
  init: RequestInit,
  sessionToken?: string
): Promise<ApiResponse<T>> {
  const headers = new Headers(init.headers);
  headers.set("Content-Type", "application/json");

  if (sessionToken) {
    headers.set("Authorization", `Bearer ${sessionToken}`);
  }

  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers
  });

  let parsed: ApiResponse<T> | null = null;
  try {
    parsed = (await response.json()) as ApiResponse<T>;
  } catch {
    if (!response.ok) {
      throw new Error(`Request failed (${response.status})`);
    }
    throw new Error("Unexpected non-JSON response");
  }

  if (!response.ok) {
    const message = parsed.error?.message ?? `Request failed (${response.status})`;
    throw new Error(message);
  }

  return parsed;
}

export type ProviderType = "ollama" | "openai" | "google";

export interface ProviderConfig {
  id: string;
  name: string;
  type: ProviderType;
  apiKey: string;
  baseUrl: string;
  modelId?: string;
  enabled: boolean;
}

export interface AgentItem {
  id: string;
  name: string;
  provider: ProviderType;
  provider_config_id?: string | null;
  model: string;
  system_prompt?: string | null;
  avatar_url?: string | null;
  skills: string[];
}

export interface ProviderSettings {
  providerConfigs: ProviderConfig[];
}

export interface ChannelMessage {
  id: string;
  sender_type: "user" | "agent";
  sender_id: string;
  sender_name: string;
  content: string;
  created_at: number;
}

export interface ChannelDetail {
  id: string;
  name: string;
  topic: string;
  users: string[];
  members: Array<{ id: string; name: string }>;
  messages: ChannelMessage[];
}

export interface ChannelSummary {
  id: string;
  name: string;
  topic: string;
  created_at: number;
  member_count: number;
  user_count: number;
}

export interface SkillItem {
  id: string;
  name: string;
  enabled: number;
}

export type ChatStreamEvent =
  | { type: "chunk"; content: string }
  | { type: "think-start"; id: string }
  | { type: "think"; id?: string; content: string }
  | { type: "think-end"; id: string }
  | { type: "tool-input-start"; toolCallId: string; toolName: string }
  | { type: "tool-input-delta"; toolCallId: string; delta: string }
  | { type: "tool-input-end"; toolCallId: string }
  | { type: "tool-call"; toolCallId: string; toolName: string; input: unknown }
  | { type: "tool-result"; toolCallId: string; toolName: string; output: unknown }
  | { type: "tool-error"; toolCallId: string; toolName: string; message: string }
  | { type: "error"; message: string }
  | { type: "final"; assistantText?: string };

export async function loginWithStartupToken(token: string): Promise<string> {
  const result = await requestJson<{ sessionToken: string }>(
    "/api/auth/login",
    {
      method: "POST",
      body: JSON.stringify({ token })
    }
  );

  if (!result.ok || !result.data?.sessionToken) {
    throw new Error(result.error?.message ?? "Login failed");
  }

  return result.data.sessionToken;
}

export async function createDevSession(): Promise<string> {
  const result = await requestJson<{ sessionToken: string }>(
    "/api/auth/dev-login",
    {
      method: "POST",
      body: JSON.stringify({})
    }
  );

  if (!result.ok || !result.data?.sessionToken) {
    throw new Error(result.error?.message ?? "Failed to create dev session");
  }

  return result.data.sessionToken;
}

export async function fetchAgents(sessionToken: string): Promise<AgentItem[]> {
  const result = await requestJson<AgentItem[]>(
    "/api/agents",
    {
      method: "GET"
    },
    sessionToken
  );

  return Array.isArray(result.data) ? result.data : [];
}

export async function createAgent(params: {
  sessionToken: string;
  name: string;
  provider?: ProviderType;
  providerConfigId?: string | null;
  model?: string;
  systemPrompt?: string;
  avatarUrl?: string | null;
  skills?: string[];
}): Promise<AgentItem> {
  const result = await requestJson<AgentItem>(
    "/api/agents",
    {
      method: "POST",
      body: JSON.stringify({
        name: params.name,
        provider: params.provider,
        providerConfigId: params.providerConfigId ?? null,
        model: params.model,
        systemPrompt: params.systemPrompt,
        avatarUrl: params.avatarUrl ?? null,
        skills: params.skills ?? []
      })
    },
    params.sessionToken
  );

  if (!result.data?.id) {
    throw new Error("Agent creation failed");
  }

  return result.data;
}

export async function updateAgent(params: {
  sessionToken: string;
  id: string;
  name: string;
  provider: ProviderType;
  providerConfigId?: string | null;
  model: string;
  systemPrompt?: string | null;
  avatarUrl?: string | null;
  skills?: string[];
}): Promise<AgentItem> {
  const result = await requestJson<AgentItem>(
    `/api/agents/${params.id}`,
    {
      method: "PUT",
      body: JSON.stringify({
        name: params.name,
        provider: params.provider,
        providerConfigId: params.providerConfigId ?? null,
        model: params.model,
        systemPrompt: params.systemPrompt ?? null,
        avatarUrl: params.avatarUrl ?? null,
        skills: params.skills ?? []
      })
    },
    params.sessionToken
  );

  if (!result.data?.id) {
    throw new Error("Agent update failed");
  }

  return result.data;
}

export async function deleteAgent(params: {
  sessionToken: string;
  id: string;
}): Promise<void> {
  await requestJson<{ id: string }>(
    `/api/agents/${params.id}`,
    { method: "DELETE" },
    params.sessionToken
  );
}

export async function fetchSkills(sessionToken: string): Promise<SkillItem[]> {
  const result = await requestJson<SkillItem[]>(
    "/api/skills",
    { method: "GET" },
    sessionToken
  );
  return Array.isArray(result.data) ? result.data : [];
}

export async function updateSkill(params: {
  sessionToken: string;
  id: string;
  enabled: boolean;
}): Promise<SkillItem> {
  const result = await requestJson<SkillItem>(
    `/api/skills/${params.id}`,
    {
      method: "PUT",
      body: JSON.stringify({ enabled: params.enabled })
    },
    params.sessionToken
  );

  if (!result.data?.id) {
    throw new Error("Failed to update skill");
  }

  return result.data;
}

export async function getProviderSettings(sessionToken: string): Promise<ProviderSettings> {
  const result = await requestJson<ProviderSettings>(
    "/api/settings/providers",
    { method: "GET" },
    sessionToken
  );

  if (!result.data) {
    throw new Error("Failed to load providers");
  }

  return result.data;
}

export async function updateProviderSettings(params: {
  sessionToken: string;
  settings: ProviderSettings;
}): Promise<ProviderSettings> {
  const result = await requestJson<ProviderSettings>(
    "/api/settings/providers",
    {
      method: "PUT",
      body: JSON.stringify(params.settings)
    },
    params.sessionToken
  );

  if (!result.data) {
    throw new Error("Failed to save providers");
  }

  return result.data;
}

export async function fetchChannels(sessionToken: string): Promise<ChannelSummary[]> {
  const result = await requestJson<ChannelSummary[]>(
    "/api/channels",
    { method: "GET" },
    sessionToken
  );
  return Array.isArray(result.data) ? result.data : [];
}

export async function getChannel(params: {
  sessionToken: string;
  id: string;
}): Promise<ChannelDetail> {
  const result = await requestJson<ChannelDetail>(
    `/api/channels/${params.id}`,
    { method: "GET" },
    params.sessionToken
  );

  if (!result.data) {
    throw new Error("Channel not found");
  }

  return result.data;
}

export async function createChannel(params: {
  sessionToken: string;
  name: string;
  topic: string;
  users: string[];
  agentIds: string[];
}): Promise<ChannelDetail> {
  const result = await requestJson<ChannelDetail>(
    "/api/channels",
    {
      method: "POST",
      body: JSON.stringify({
        name: params.name,
        topic: params.topic,
        users: params.users,
        agentIds: params.agentIds
      })
    },
    params.sessionToken
  );

  if (!result.data) {
    throw new Error("Failed to create channel");
  }

  return result.data;
}

export async function updateChannel(params: {
  sessionToken: string;
  id: string;
  name: string;
  topic: string;
  users: string[];
  agentIds: string[];
}): Promise<ChannelDetail> {
  const result = await requestJson<ChannelDetail>(
    `/api/channels/${params.id}`,
    {
      method: "PUT",
      body: JSON.stringify({
        name: params.name,
        topic: params.topic,
        users: params.users,
        agentIds: params.agentIds
      })
    },
    params.sessionToken
  );

  if (!result.data) {
    throw new Error("Failed to update channel");
  }

  return result.data;
}

export async function deleteChannel(params: {
  sessionToken: string;
  id: string;
}): Promise<void> {
  await requestJson(`/api/channels/${params.id}`, { method: "DELETE" }, params.sessionToken);
}

export async function sendChannelMessage(params: {
  sessionToken: string;
  channelId: string;
  message: string;
  userName?: string;
}): Promise<ChannelDetail> {
  const result = await requestJson<{ channel: ChannelDetail }>(
    `/api/channels/${params.channelId}/chat`,
    {
      method: "POST",
      body: JSON.stringify({
        message: params.message,
        userName: params.userName ?? "You"
      })
    },
    params.sessionToken
  );

  if (!result.data?.channel) {
    throw new Error("Failed to send message");
  }

  return result.data.channel;
}

export async function getOllamaSettings(sessionToken: string): Promise<{ baseUrl: string }> {
  const result = await requestJson<{ baseUrl: string }>(
    "/api/settings/ollama",
    { method: "GET" },
    sessionToken
  );

  return result.data ?? { baseUrl: "http://127.0.0.1:11434" };
}

export async function updateOllamaSettings(params: {
  sessionToken: string;
  baseUrl: string;
}): Promise<{ baseUrl: string }> {
  const result = await requestJson<{ baseUrl: string }>(
    "/api/settings/ollama",
    {
      method: "PUT",
      body: JSON.stringify({ baseUrl: params.baseUrl })
    },
    params.sessionToken
  );

  if (!result.data) {
    throw new Error("Failed to save settings");
  }

  return result.data;
}

export async function streamChat(params: {
  sessionToken: string;
  agentId: string;
  message: string;
  onChunk: (chunk: string) => void;
  onEvent?: (event: ChatStreamEvent) => void;
}): Promise<void> {
  const response = await fetch(`${API_BASE}/api/chat`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${params.sessionToken}`
    },
    body: JSON.stringify({
      agentId: params.agentId,
      message: params.message
    })
  });

  if (!response.ok || !response.body) {
    throw new Error(`Chat request failed: ${response.status}`);
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }

    buffer += decoder.decode(value, { stream: true });
    const events = buffer.split("\n\n");
    buffer = events.pop() ?? "";

    for (const eventText of events) {
      const line = eventText
        .split("\n")
        .find((entry) => entry.startsWith("data: "));

      if (!line) {
        continue;
      }

      const data = line.slice("data: ".length);
      if (data === "[DONE]") {
        return;
      }

      let parsed:
        | { type?: string; content?: string; message?: string }
        | undefined;
      try {
        parsed = JSON.parse(data) as {
          type?: string;
          content?: string;
          message?: string;
        };
      } catch {
        continue;
      }

      if (parsed.type) {
        params.onEvent?.(parsed as ChatStreamEvent);
      }

      if (parsed.type === "chunk" && parsed.content) {
        params.onChunk(parsed.content);
      } else if (parsed.type === "error" && parsed.message) {
        throw new Error(parsed.message);
      }
    }
  }
}
