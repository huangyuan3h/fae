export const API_BASE =
  process.env.NEXT_PUBLIC_API_BASE ?? "http://127.0.0.1:8080";

export interface LoginResponse {
  ok: boolean;
  data?: {
    sessionToken: string;
  };
  error?: {
    code: string;
    message: string;
  };
}

export async function loginWithStartupToken(token: string): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE}/api/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ token })
  });
  return (await response.json()) as LoginResponse;
}

export async function fetchAgents(sessionToken: string) {
  const response = await fetch(`${API_BASE}/api/agents`, {
    headers: {
      Authorization: `Bearer ${sessionToken}`
    }
  });
  return response.json();
}

export async function getOllamaSettings(sessionToken: string) {
  const response = await fetch(`${API_BASE}/api/settings/ollama`, {
    headers: {
      Authorization: `Bearer ${sessionToken}`
    }
  });
  return response.json();
}

export async function updateOllamaSettings(params: {
  sessionToken: string;
  baseUrl: string;
}) {
  const response = await fetch(`${API_BASE}/api/settings/ollama`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${params.sessionToken}`
    },
    body: JSON.stringify({ baseUrl: params.baseUrl })
  });
  return response.json();
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

      try {
        const parsed = JSON.parse(data) as { type?: string; content?: string };
        if (parsed.type === "chunk" && parsed.content) {
          params.onChunk(parsed.content);
        }
      } catch {
        // Ignore malformed SSE chunks in MVP stage.
      }
    }
  }
}
