"use client";

import { useEffect, useMemo, useState } from "react";
import type { FormEvent } from "react";
import { useRouter } from "next/navigation";
import { fetchAgents, streamChat } from "../../lib/api";

const SESSION_KEY = "fae_session_token";

interface AgentItem {
  id: string;
  name: string;
}

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

export default function ChatPage() {
  const router = useRouter();
  const [sessionToken, setSessionToken] = useState<string>("");
  const [agents, setAgents] = useState<AgentItem[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState("");
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const token = localStorage.getItem(SESSION_KEY);
    if (!token) {
      router.replace("/login");
      return;
    }
    setSessionToken(token);
  }, [router]);

  useEffect(() => {
    if (!sessionToken) {
      return;
    }
    fetchAgents(sessionToken)
      .then((result) => {
        const list = Array.isArray(result?.data) ? (result.data as AgentItem[]) : [];
        setAgents(list);
        if (list.length > 0) {
          setSelectedAgentId(list[0].id);
        }
      })
      .catch(() => {
        localStorage.removeItem(SESSION_KEY);
        router.replace("/login");
      });
  }, [sessionToken, router]);

  const canSubmit = useMemo(
    () => Boolean(sessionToken && selectedAgentId && input.trim() && !loading),
    [sessionToken, selectedAgentId, input, loading]
  );

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) {
      return;
    }

    const message = input.trim();
    setInput("");
    setLoading(true);
    setMessages((prev) => [...prev, { role: "user", content: message }, { role: "assistant", content: "" }]);

    try {
      await streamChat({
        sessionToken,
        agentId: selectedAgentId,
        message,
        onChunk: (chunk) => {
          setMessages((prev) => {
            const next = [...prev];
            const lastIdx = next.length - 1;
            if (lastIdx >= 0 && next[lastIdx].role === "assistant") {
              next[lastIdx] = {
                role: "assistant",
                content: next[lastIdx].content + chunk
              };
            }
            return next;
          });
        }
      });
    } catch (error) {
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content: error instanceof Error ? error.message : "Chat failed"
        }
      ]);
    } finally {
      setLoading(false);
    }
  }

  return (
    <main>
      <h1>fae Chat</h1>
      <div style={{ display: "grid", gap: 12 }}>
        <label>
          Agent
          <select
            value={selectedAgentId}
            onChange={(event) => setSelectedAgentId(event.target.value)}
            style={{ marginLeft: 8 }}
          >
            {agents.map((agent) => (
              <option key={agent.id} value={agent.id}>
                {agent.name}
              </option>
            ))}
          </select>
        </label>

        <div
          style={{
            minHeight: 260,
            border: "1px solid #666",
            borderRadius: 8,
            padding: 12,
            display: "grid",
            gap: 8
          }}
        >
          {messages.map((message, idx) => (
            <div key={`${idx}-${message.role}`}>
              <strong>{message.role}:</strong> {message.content}
            </div>
          ))}
        </div>

        <form onSubmit={onSubmit} style={{ display: "grid", gap: 8 }}>
          <textarea
            value={input}
            onChange={(event) => setInput(event.target.value)}
            placeholder="Ask your agent..."
            rows={4}
          />
          <button type="submit" disabled={!canSubmit}>
            {loading ? "Streaming..." : "Send"}
          </button>
        </form>
      </div>
    </main>
  );
}
