"use client";

import type { FormEvent } from "react";
import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowUp, Bot, Sparkle, User } from "lucide-react";
import { useRouter } from "next/navigation";
import { AppShell } from "../../components/app-shell";
import { Alert, AlertDescription, AlertTitle } from "../../components/ui/alert";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle
} from "../../components/ui/card";
import { Label } from "../../components/ui/label";
import { Select } from "../../components/ui/select";
import { Spinner } from "../../components/ui/spinner";
import { Textarea } from "../../components/ui/textarea";
import { createAgent, fetchAgents, streamChat, type AgentItem } from "../../lib/api";
import { clearSessionToken, getSessionToken } from "../../lib/session";
import { cn } from "../../lib/utils";

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
  const [bootstrapping, setBootstrapping] = useState(true);
  const [statusText, setStatusText] = useState<string>("");
  const [pageError, setPageError] = useState<string>("");
  const messageEndRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const token = getSessionToken();
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

    let active = true;

    async function loadAgents(): Promise<void> {
      setBootstrapping(true);
      setPageError("");
      setStatusText("Loading agents...");

      try {
        const list = await fetchAgents(sessionToken);
        if (!active) {
          return;
        }

        if (list.length > 0) {
          setAgents(list);
          setSelectedAgentId((prev) => prev || list[0].id);
          setStatusText(`Loaded ${list.length} agent${list.length > 1 ? "s" : ""}.`);
          return;
        }

        setStatusText("No agents found. Creating default agent...");
        const createdAgent = await createAgent({
          sessionToken,
          name: "My Local Assistant",
          model: "qwen3.5:27b",
          systemPrompt: "You are a helpful local AI assistant."
        });

        if (!active) {
          return;
        }

        setAgents([createdAgent]);
        setSelectedAgentId(createdAgent.id);
        setStatusText("Default agent created.");
      } catch (error) {
        if (!active) {
          return;
        }

        clearSessionToken();
        setPageError(error instanceof Error ? error.message : "Failed to load chat.");
      } finally {
        if (active) {
          setBootstrapping(false);
        }
      }
    }

    void loadAgents();

    return () => {
      active = false;
    };
  }, [sessionToken]);

  useEffect(() => {
    messageEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const canSubmit = useMemo(
    () => Boolean(sessionToken && selectedAgentId && input.trim() && !loading),
    [sessionToken, selectedAgentId, input, loading]
  );

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) {
      return;
    }

    setPageError("");

    const message = input.trim();
    setInput("");
    setLoading(true);
    setMessages((prev) => [
      ...prev,
      { role: "user", content: message },
      { role: "assistant", content: "" }
    ]);

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
      const errorText = error instanceof Error ? error.message : "Chat failed";
      setPageError(errorText);
      setMessages((prev) => {
        const next = [...prev];
        const lastIdx = next.length - 1;
        if (lastIdx >= 0 && next[lastIdx].role === "assistant") {
          next[lastIdx] = { role: "assistant", content: `Error: ${errorText}` };
        }
        return next;
      });
    } finally {
      setLoading(false);
    }
  }

  if (pageError && !sessionToken) {
    return null;
  }

  return (
    <AppShell
      active="chat"
      title="Chat Workspace"
      subtitle="Talk to your local agent with streaming responses."
    >
      <section className="grid gap-4 lg:grid-cols-[320px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Session</CardTitle>
            <CardDescription>Agent selection and status.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="agent-select">Agent</Label>
              <Select
                id="agent-select"
                value={selectedAgentId}
                onChange={(event) => setSelectedAgentId(event.target.value)}
                disabled={bootstrapping || agents.length === 0}
                options={agents.map((agent) => ({
                  value: agent.id,
                  label: `${agent.name} (${agent.model ?? "unknown model"})`
                }))}
              />
            </div>

            <Badge variant={bootstrapping ? "secondary" : "success"}>
              {bootstrapping ? "Preparing workspace..." : statusText || "Ready"}
            </Badge>

            {pageError ? (
              <Alert variant="destructive">
                <AlertTitle>Chat unavailable</AlertTitle>
                <AlertDescription>{pageError}</AlertDescription>
              </Alert>
            ) : null}
          </CardContent>
        </Card>

        <Card className="overflow-hidden">
          <CardHeader className="border-b border-slate-800/70">
            <CardTitle className="text-lg">Conversation</CardTitle>
            <CardDescription>
              Message history stays in this session tab.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex h-[68vh] flex-col gap-4 p-4 sm:p-5">
            <div className="flex-1 space-y-3 overflow-y-auto pr-1">
              {messages.length === 0 ? (
                <div className="rounded-xl border border-dashed border-slate-700 bg-slate-900/40 p-4 text-sm text-slate-400">
                  Send your first message to start streaming from your local model.
                </div>
              ) : null}

              {messages.map((message, idx) => (
                <article
                  key={`${idx}-${message.role}`}
                  className={cn(
                    "max-w-[92%] rounded-2xl border p-3 text-sm leading-relaxed",
                    message.role === "assistant"
                      ? "border-slate-700 bg-slate-900/75 text-slate-100"
                      : "ml-auto border-sky-300/35 bg-sky-400/15 text-sky-50"
                  )}
                >
                  <header className="mb-1 inline-flex items-center gap-1.5 text-xs font-medium uppercase tracking-[0.14em] text-slate-400">
                    {message.role === "assistant" ? (
                      <Bot className="h-3.5 w-3.5" />
                    ) : (
                      <User className="h-3.5 w-3.5" />
                    )}
                    {message.role}
                  </header>
                  <p className="whitespace-pre-wrap break-words">{message.content}</p>
                </article>
              ))}
              <div ref={messageEndRef} />
            </div>

            <form onSubmit={onSubmit} className="space-y-3 border-t border-slate-800/70 pt-3">
              <Label htmlFor="prompt">Prompt</Label>
              <Textarea
                id="prompt"
                value={input}
                onChange={(event) => setInput(event.target.value)}
                placeholder="Ask your agent..."
                rows={4}
                disabled={bootstrapping}
              />
              <div className="flex items-center justify-between">
                <p className="inline-flex items-center gap-1.5 text-xs text-slate-500">
                  <Sparkle className="h-3.5 w-3.5" />
                  Streaming from daemon
                </p>
                <Button type="submit" disabled={!canSubmit}>
                  {loading ? <Spinner className="h-4 w-4" /> : <ArrowUp className="h-4 w-4" />}
                  {loading ? "Streaming..." : "Send"}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </section>
    </AppShell>
  );
}
