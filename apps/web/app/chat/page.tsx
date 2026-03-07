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

function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/).slice(0, 2);
  if (parts.length === 0 || !parts[0]) {
    return "FA";
  }
  return parts.map((part) => part[0]?.toUpperCase() ?? "").join("");
}

function isImageAvatar(value: string): boolean {
  return value.startsWith("data:image/") || value.startsWith("http://") || value.startsWith("https://");
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

  // 获取URL参数中的agent值
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const urlParams = new URLSearchParams(window.location.search);
      const agentFromUrl = urlParams.get('agent');
      if (agentFromUrl) {
        // 延迟设置，确保agents已加载后再设置（避免UI显示不一致）
        setTimeout(() => {
          setSelectedAgentId(agentFromUrl);
        }, 0);
      }
    }
  }, []);

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
          // 如果已经设置了selectedAgentId（来自URL参数），则使用它；否则使用第一个agent
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

  const selectedAgent = agents.find(agent => agent.id === selectedAgentId);
  
  return (
    <AppShell
      active="chat"
      title="Chat Workspace"
      subtitle="Streaming text, thinking traces, and tool execution events."
    >
      <section className="grid gap-4 lg:grid-cols-[320px_1fr]">
        <Card>
          {selectedAgent ? (
            <>
              <CardHeader>
                <CardTitle className="text-lg flex items-center gap-2">
                  <span className="inline-block w-3 h-3 rounded-full bg-green-500"></span>
                  {selectedAgent.name}
                </CardTitle>
                <CardDescription>Digital employee details</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center gap-3">
                  <div className="relative">
                    {selectedAgent.avatar_url?.startsWith("emoji:") ? (
                      <div className="inline-flex h-12 w-12 items-center justify-center rounded-full border border-blue-200 bg-blue-50 text-2xl">
                        {selectedAgent.avatar_url.slice(6)}
                      </div>
                    ) : selectedAgent.avatar_url && (selectedAgent.avatar_url.startsWith("data:image/") || selectedAgent.avatar_url.startsWith("http://") || selectedAgent.avatar_url.startsWith("https://")) ? (
                      <img
                        src={selectedAgent.avatar_url}
                        alt={`${selectedAgent.name} avatar`}
                        className="h-12 w-12 rounded-full border border-slate-200 object-cover"
                      />
                    ) : (
                      <div className="inline-flex h-12 w-12 items-center justify-center rounded-full border border-blue-200 bg-blue-50 text-sm font-semibold text-blue-700">
                        {getInitials(selectedAgent.name)}
                      </div>
                    )}
                  </div>
                  <div>
                    <h3 className="font-medium text-slate-900">{selectedAgent.name}</h3>
                    <p className="text-sm text-slate-500">{selectedAgent.provider} • {selectedAgent.model}</p>
                  </div>
                </div>
                
                <div className="space-y-2">
                  <Label className="text-xs font-medium text-slate-700">System Prompt</Label>
                  <div className="rounded-lg border border-slate-200 bg-slate-50 p-3 text-xs text-slate-600 max-h-32 overflow-y-auto">
                    <p className="whitespace-pre-wrap">{selectedAgent.system_prompt || "(no prompt)"}</p>
                  </div>
                </div>
                
                <div className="space-y-2">
                  <Label className="text-xs font-medium text-slate-700">Skills</Label>
                  <div className="flex flex-wrap gap-1">
                    {selectedAgent.skills && selectedAgent.skills.length > 0 ? (
                      selectedAgent.skills.map((skillId) => (
                        <Badge key={skillId} variant="secondary" className="text-xs">
                          {skillId}
                        </Badge>
                      ))
                    ) : (
                      <Badge variant="secondary" className="text-xs opacity-50">
                        No skills assigned
                      </Badge>
                    )}
                  </div>
                </div>
              </CardContent>
            </>
          ) : (
            <>
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
            </>
          )}
        </Card>

        <Card className="overflow-hidden">
          <CardHeader className="border-b border-slate-200">
            <CardTitle className="text-lg">Conversation</CardTitle>
            <CardDescription>
              Includes reasoning and tool-call traces from streaming events.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex h-[68vh] flex-col gap-4 p-4 sm:p-5">
            <div className="flex-1 space-y-3 overflow-y-auto pr-1">
              {messages.length === 0 ? (
                <div className="rounded-xl border border-dashed border-slate-300 bg-slate-50 p-4 text-sm text-slate-500">
                  Ask something to see stream output, thinking traces, and tool execution.
                </div>
              ) : null}

              {messages.map((message, idx) => (
                <article
                  key={`${idx}-${message.role}`}
                  className={cn(
                    "max-w-[92%] rounded-2xl border p-3 text-sm leading-relaxed",
                    message.role === "assistant"
                      ? "border-slate-200 bg-white text-slate-900"
                      : "ml-auto border-blue-200 bg-blue-50 text-blue-900"
                  )}
                >
                  <header className="mb-1 inline-flex items-center gap-1.5 text-xs font-medium uppercase tracking-[0.14em] text-slate-500">
                    {message.role === "assistant" ? (
                      <Bot className="h-3.5 w-3.5" />
                    ) : (
                      <User className="h-3.5 w-3.5" />
                    )}
                    {message.role}
                  </header>

                  {message.role === "assistant" && message.thinking.trim().length > 0 ? (
                    <details className="mb-2 rounded-lg border border-slate-200 bg-slate-50 p-2 text-xs text-slate-600">
                      <summary className="flex cursor-pointer list-none items-center gap-1.5 font-medium text-slate-700">
                        <Brain className="h-3.5 w-3.5" />
                        Thinking Trace
                      </summary>
                      <pre className="mt-2 whitespace-pre-wrap font-[family-name:var(--font-ibm-plex-mono)] text-[11px] leading-relaxed text-slate-600">
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
                          className="rounded-lg border border-slate-200 bg-slate-50 p-2"
                        >
                          <div className="mb-1 flex items-center justify-between gap-2 text-xs">
                            <p className="inline-flex items-center gap-1.5 font-medium text-slate-700">
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
                            <pre className="mb-2 whitespace-pre-wrap rounded-md bg-white p-2 font-[family-name:var(--font-ibm-plex-mono)] text-[11px] text-slate-600 border border-slate-200">
                              {trace.inputText}
                            </pre>
                          ) : null}

                          {trace.outputText ? (
                            <pre className="whitespace-pre-wrap rounded-md bg-emerald-50 p-2 font-[family-name:var(--font-ibm-plex-mono)] text-[11px] text-emerald-700 border border-emerald-200">
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

            <form onSubmit={onSubmit} className="space-y-3 border-t border-slate-200 pt-3">
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
