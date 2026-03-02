"use client";

import type { FormEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import { AppShell } from "../../components/app-shell";
import { Alert, AlertDescription, AlertTitle } from "../../components/ui/alert";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui/card";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Select } from "../../components/ui/select";
import { Spinner } from "../../components/ui/spinner";
import { Textarea } from "../../components/ui/textarea";
import {
  createChannel,
  createDevSession,
  deleteChannel,
  fetchAgents,
  fetchChannels,
  getChannel,
  sendChannelMessage,
  type AgentItem,
  type ChannelDetail,
  type ChannelSummary
} from "../../lib/api";
import { ensureSessionToken } from "../../lib/session";

function parseCsv(value: string): string[] {
  return value
    .split(",")
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
}

export default function ChannelsPage() {
  const [sessionToken, setSessionToken] = useState("");
  const [channels, setChannels] = useState<ChannelSummary[]>([]);
  const [agents, setAgents] = useState<AgentItem[]>([]);
  const [selectedChannelId, setSelectedChannelId] = useState("");
  const [selectedChannel, setSelectedChannel] = useState<ChannelDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");

  const [channelName, setChannelName] = useState("");
  const [channelTopic, setChannelTopic] = useState("");
  const [channelUsers, setChannelUsers] = useState("Alice,Bob");
  const [channelAgentIds, setChannelAgentIds] = useState<string[]>([]);
  const [messageInput, setMessageInput] = useState("");

  useEffect(() => {
    ensureSessionToken(createDevSession)
      .then((token) => setSessionToken(token))
      .catch(() => setError("Failed to create development session."));
  }, []);

  useEffect(() => {
    if (!sessionToken) {
      return;
    }

    setLoading(true);
    Promise.all([fetchChannels(sessionToken), fetchAgents(sessionToken)])
      .then(([channelRows, agentRows]) => {
        setChannels(channelRows);
        setAgents(agentRows);
        const initial = channelRows[0]?.id ?? "";
        setSelectedChannelId(initial);
        setChannelAgentIds((prev) => (prev.length > 0 ? prev : agentRows.slice(0, 1).map((a) => a.id)));
      })
      .catch((loadError) => {
        setError(loadError instanceof Error ? loadError.message : "Failed to load channels");
      })
      .finally(() => setLoading(false));
  }, [sessionToken]);

  useEffect(() => {
    if (!sessionToken || !selectedChannelId) {
      setSelectedChannel(null);
      return;
    }

    getChannel({ sessionToken, id: selectedChannelId })
      .then((channel) => setSelectedChannel(channel))
      .catch((loadError) => {
        setError(loadError instanceof Error ? loadError.message : "Failed to load channel details");
      });
  }, [sessionToken, selectedChannelId]);

  const channelOptions = useMemo(
    () => channels.map((channel) => ({ value: channel.id, label: channel.name })),
    [channels]
  );

  async function onCreateChannel(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!channelName.trim()) {
      return;
    }

    setCreating(true);
    setError("");
    try {
      const created = await createChannel({
        sessionToken,
        name: channelName.trim(),
        topic: channelTopic.trim(),
        users: parseCsv(channelUsers),
        agentIds: channelAgentIds
      });

      setChannels((prev) => [
        {
          id: created.id,
          name: created.name,
          topic: created.topic,
          created_at: Date.now(),
          member_count: created.members.length,
          user_count: created.users.length
        },
        ...prev
      ]);
      setSelectedChannelId(created.id);
      setSelectedChannel(created);
      setChannelName("");
      setChannelTopic("");
    } catch (createError) {
      setError(createError instanceof Error ? createError.message : "Failed to create channel");
    } finally {
      setCreating(false);
    }
  }

  async function onDeleteChannel(channelId: string) {
    try {
      await deleteChannel({ sessionToken, id: channelId });
      const next = channels.filter((channel) => channel.id !== channelId);
      setChannels(next);
      setSelectedChannelId(next[0]?.id ?? "");
    } catch (deleteError) {
      setError(deleteError instanceof Error ? deleteError.message : "Failed to delete channel");
    }
  }

  async function onSendMessage(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedChannelId || !messageInput.trim()) {
      return;
    }

    setSending(true);
    setError("");
    try {
      const updated = await sendChannelMessage({
        sessionToken,
        channelId: selectedChannelId,
        message: messageInput.trim(),
        userName: "You"
      });
      setSelectedChannel(updated);
      setMessageInput("");
    } catch (sendError) {
      setError(sendError instanceof Error ? sendError.message : "Failed to send message");
    } finally {
      setSending(false);
    }
  }

  return (
    <AppShell
      active="channels"
      title="Channels"
      subtitle="Manage users, topics, and employee members. Chat inside each channel."
    >
      <section className="grid gap-4 lg:grid-cols-[360px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Create Channel</CardTitle>
            <CardDescription>
              Add topic, users, and participating digital employees.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={onCreateChannel} className="grid gap-3">
              <div className="grid gap-2">
                <Label htmlFor="channel-name">Name</Label>
                <Input
                  id="channel-name"
                  value={channelName}
                  onChange={(event) => setChannelName(event.target.value)}
                  placeholder="trading-desk"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="channel-topic">Topic</Label>
                <Input
                  id="channel-topic"
                  value={channelTopic}
                  onChange={(event) => setChannelTopic(event.target.value)}
                  placeholder="Daily market analysis"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="channel-users">Users (comma-separated)</Label>
                <Input
                  id="channel-users"
                  value={channelUsers}
                  onChange={(event) => setChannelUsers(event.target.value)}
                  placeholder="Alice,Bob"
                />
              </div>

              <div className="grid gap-2">
                <Label>Employees</Label>
                <div className="max-h-40 space-y-2 overflow-y-auto rounded-lg border border-slate-200 p-2">
                  {agents.map((agent) => (
                    <label key={agent.id} className="flex items-center gap-2 text-sm text-slate-700">
                      <input
                        type="checkbox"
                        checked={channelAgentIds.includes(agent.id)}
                        onChange={(event) => {
                          setChannelAgentIds((prev) =>
                            event.target.checked
                              ? [...prev, agent.id]
                              : prev.filter((id) => id !== agent.id)
                          );
                        }}
                      />
                      {agent.name} ({agent.provider})
                    </label>
                  ))}
                </div>
              </div>

              <Button type="submit" disabled={creating || !sessionToken}>
                {creating ? <Spinner className="h-4 w-4" /> : null}
                {creating ? "Creating..." : "Create Channel"}
              </Button>
            </form>

            <div className="mt-4 space-y-2">
              <Label htmlFor="channel-select">Existing channels</Label>
              <Select
                id="channel-select"
                value={selectedChannelId}
                onChange={(event) => setSelectedChannelId(event.target.value)}
                options={
                  channelOptions.length > 0
                    ? channelOptions
                    : [{ value: "", label: "No channels yet" }]
                }
              />
              {selectedChannelId ? (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onDeleteChannel(selectedChannelId)}
                >
                  Delete Channel
                </Button>
              ) : null}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{selectedChannel?.name ?? "Channel Chat"}</CardTitle>
            <CardDescription>{selectedChannel?.topic ?? "Select a channel"}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {loading ? (
              <p className="text-sm text-slate-500">Loading channels...</p>
            ) : selectedChannel ? (
              <>
                <div className="flex flex-wrap gap-1">
                  {selectedChannel.users.map((user) => (
                    <Badge key={user} variant="secondary">
                      {user}
                    </Badge>
                  ))}
                  {selectedChannel.members.map((member) => (
                    <Badge key={member.id} variant="default">
                      {member.name}
                    </Badge>
                  ))}
                </div>

                <div className="max-h-[50vh] space-y-2 overflow-y-auto rounded-lg border border-slate-200 bg-white p-3">
                  {selectedChannel.messages.length === 0 ? (
                    <p className="text-sm text-slate-500">No messages yet.</p>
                  ) : (
                    selectedChannel.messages.map((message) => (
                      <div key={message.id} className="rounded-md border border-slate-100 p-2">
                        <p className="text-xs font-semibold text-slate-700">
                          {message.sender_name} · {message.sender_type}
                        </p>
                        <p className="whitespace-pre-wrap text-sm text-slate-800">{message.content}</p>
                      </div>
                    ))
                  )}
                </div>

                <form onSubmit={onSendMessage} className="space-y-2">
                  <Textarea
                    value={messageInput}
                    onChange={(event) => setMessageInput(event.target.value)}
                    rows={4}
                    placeholder="Message this channel..."
                  />
                  <Button type="submit" disabled={sending || !selectedChannelId}>
                    {sending ? <Spinner className="h-4 w-4" /> : null}
                    {sending ? "Sending..." : "Send"}
                  </Button>
                </form>
              </>
            ) : (
              <p className="text-sm text-slate-500">Create or select a channel.</p>
            )}

            {error ? (
              <Alert variant="destructive">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            ) : null}
          </CardContent>
        </Card>
      </section>
    </AppShell>
  );
}
