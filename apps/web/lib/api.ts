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

export interface AgentItem {
  id: string;
  name: string;
  model?: string;
}

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
  model?: string;
  systemPrompt?: string;
}): Promise<AgentItem> {
  const result = await requestJson<AgentItem>(
    "/api/agents",
    {
      method: "POST",
      body: JSON.stringify({
        name: params.name,
        model: params.model,
        systemPrompt: params.systemPrompt
      })
    },
    params.sessionToken
  );

  if (!result.data?.id) {
    throw new Error("Agent creation failed");
  }

  return result.data;
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

      if (parsed.type === "chunk" && parsed.content) {
        params.onChunk(parsed.content);
      } else if (parsed.type === "error" && parsed.message) {
        throw new Error(parsed.message);
      }
    }
  }
}
