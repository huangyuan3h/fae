"use client";

import type { FormEvent } from "react";
import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowUp, Bot, Brain, Sparkle, User, Wrench } from "lucide-react";
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
import {
  createAgent,
  createDevSession,
  fetchAgents,
  streamChat,
  type AgentItem,
  type ChatStreamEvent
} from "../../lib/api";
import { clearSessionToken, ensureSessionToken } from "../../lib/session";
import { cn } from "../../lib/utils";

interface ToolTrace {
  toolCallId: string;
  toolName: string;
  status: "input" | "called" | "done" | "error";
  inputText: string;
  outputText: string;
  errorText: string;
}

type ChatMessage =
  | {
      role: "user";
      content: string;
    }
  | {
      role: "assistant";
      content: string;
      thinking: string;
      toolTraces: ToolTrace[];
    };

function stringifyValue(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function upsertTrace(
  traces: ToolTrace[],
  toolCallId: string,
  apply: (trace: ToolTrace) => ToolTrace,
  fallbackName = "tool"
): ToolTrace[] {
  const index = traces.findIndex((trace) => trace.toolCallId === toolCallId);
  if (index < 0) {
    const created = apply({
      toolCallId,
      toolName: fallbackName,
      status: "input",
      inputText: "",
      outputText: "",
      errorText: ""
    });
    return [...traces, created];
  }

  const next = [...traces];
  next[index] = apply(next[index]);
  return next;
}

function applyStreamEvent(message: ChatMessage, event: ChatStreamEvent): ChatMessage {
  if (message.role !== "assistant") {
    return message;
  }

  if (event.type === "think") {
    return {
      ...message,
      thinking: message.thinking + event.content
    };
  }

  if (event.type === "tool-input-start") {
    return {
      ...message,
      toolTraces: upsertTrace(
        message.toolTraces,
        event.toolCallId,
        (trace) => ({ ...trace, toolName: event.toolName, status: "input" }),
        event.toolName
      )
    };
  }

  if (event.type === "tool-input-delta") {
    return {
      ...message,
      toolTraces: upsertTrace(message.toolTraces, event.toolCallId, (trace) => ({
        ...trace,
        status: "input",
        inputText: trace.inputText + event.delta
      }))
    };
  }

  if (event.type === "tool-call") {
    return {
      ...message,
      toolTraces: upsertTrace(
        message.toolTraces,
        event.toolCallId,
        (trace) => ({
          ...trace,
          toolName: event.toolName,
          status: "called",
          inputText: stringifyValue(event.input) || trace.inputText
        }),
        event.toolName
      )
    };
  }

  if (event.type === "tool-result") {
    return {
      ...message,
      toolTraces: upsertTrace(
        message.toolTraces,
        event.toolCallId,
        (trace) => ({
          ...trace,
          toolName: event.toolName,
          status: "done",
          outputText: stringifyValue(event.output)
        }),
        event.toolName
      )
    };
  }

  if (event.type === "tool-error") {
    return {
      ...message,
      toolTraces: upsertTrace(
        message.toolTraces,
        event.toolCallId,
        (trace) => ({
          ...trace,
          toolName: event.toolName,
          status: "error",
          errorText: event.message
        }),
        event.toolName
      )
    };
  }

  return message;
}

export default function ChatPage() {
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
  const sessionRetryRef = useRef(false);

  useEffect(() => {
    ensureSessionToken(createDevSession)
      .then((token) => {
        setSessionToken(token);
      })
      .catch(() => {
        setPageError("Failed to create development session.");
      });
  }, []);

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
          systemPrompt:
            "You are a helpful local AI assistant. Use tools when they are useful and explain your process clearly."
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
        if (!sessionRetryRef.current) {
          sessionRetryRef.current = true;
          try {
            const renewedToken = await createDevSession();
            if (active) {
              setSessionToken(renewedToken);
              setStatusText("Session renewed.");
              return;
            }
          } catch {
            // Fall through to error state below.
          }
        }

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

    const prompt = input.trim();
    setInput("");
    setLoading(true);

    setMessages((prev) => [
      ...prev,
      { role: "user", content: prompt },
      { role: "assistant", content: "", thinking: "", toolTraces: [] }
    ]);

    try {
      await streamChat({
        sessionToken,
        agentId: selectedAgentId,
        message: prompt,
        onChunk: (chunk) => {
          setMessages((prev) => {
            const next = [...prev];
            const last = next[next.length - 1];
            if (last?.role === "assistant") {
              next[next.length - 1] = {
                ...last,
                content: last.content + chunk
              };
            }
            return next;
          });
        },
        onEvent: (streamEvent) => {
          setMessages((prev) => {
            const next = [...prev];
            const last = next[next.length - 1];
            if (!last || last.role !== "assistant") {
              return prev;
            }
            next[next.length - 1] = applyStreamEvent(last, streamEvent);
            return next;
          });
        }
      });
    } catch (error) {
      const errorText = error instanceof Error ? error.message : "Chat failed";
      setPageError(errorText);
      setMessages((prev) => {
        const next = [...prev];
        const last = next[next.length - 1];
        if (last?.role === "assistant") {
          next[next.length - 1] = { ...last, content: `Error: ${errorText}` };
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
      subtitle="Streaming text, thinking traces, and tool execution events."
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
              Includes reasoning and tool-call traces from streaming events.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex h-[68vh] flex-col gap-4 p-4 sm:p-5">
            <div className="flex-1 space-y-3 overflow-y-auto pr-1">
              {messages.length === 0 ? (
                <div className="rounded-xl border border-dashed border-slate-700 bg-slate-900/40 p-4 text-sm text-slate-400">
                  Ask something to see stream output, thinking traces, and tool execution.
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

                  {message.role === "assistant" && message.thinking.trim().length > 0 ? (
                    <details className="mb-2 rounded-lg border border-slate-700/80 bg-slate-950/70 p-2 text-xs text-slate-300">
                      <summary className="flex cursor-pointer list-none items-center gap-1.5 font-medium text-slate-200">
                        <Brain className="h-3.5 w-3.5" />
                        Thinking Trace
                      </summary>
                      <pre className="mt-2 whitespace-pre-wrap font-[family-name:var(--font-ibm-plex-mono)] text-[11px] leading-relaxed text-slate-300">
                        {message.thinking}
                      </pre>
                    </details>
                  ) : null}

                  <p className="whitespace-pre-wrap break-words">{message.content}</p>

                  {message.role === "assistant" && message.toolTraces.length > 0 ? (
                    <div className="mt-3 space-y-2">
                      {message.toolTraces.map((trace) => (
                        <section
                          key={trace.toolCallId}
                          className="rounded-lg border border-slate-700/80 bg-slate-950/70 p-2"
                        >
                          <div className="mb-1 flex items-center justify-between gap-2 text-xs">
                            <p className="inline-flex items-center gap-1.5 font-medium text-slate-200">
                              <Wrench className="h-3.5 w-3.5" />
                              {trace.toolName}
                            </p>
                            <Badge
                              variant={
                                trace.status === "error"
                                  ? "danger"
                                  : trace.status === "done"
                                    ? "success"
                                    : "secondary"
                              }
                            >
                              {trace.status}
                            </Badge>
                          </div>

                          {trace.inputText ? (
                            <pre className="mb-2 whitespace-pre-wrap rounded-md bg-slate-900 p-2 font-[family-name:var(--font-ibm-plex-mono)] text-[11px] text-slate-300">
                              {trace.inputText}
                            </pre>
                          ) : null}

                          {trace.outputText ? (
                            <pre className="whitespace-pre-wrap rounded-md bg-slate-900 p-2 font-[family-name:var(--font-ibm-plex-mono)] text-[11px] text-emerald-200">
                              {trace.outputText}
                            </pre>
                          ) : null}

                          {trace.errorText ? (
                            <p className="text-xs text-rose-300">{trace.errorText}</p>
                          ) : null}
                        </section>
                      ))}
                    </div>
                  ) : null}
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
                  AI SDK streaming enabled
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
