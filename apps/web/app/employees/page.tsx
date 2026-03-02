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
  createAgent,
  createDevSession,
  deleteAgent,
  fetchAgents,
  fetchSkills,
  type AgentItem,
  type ProviderType
} from "../../lib/api";
import { ensureSessionToken } from "../../lib/session";

const providerModels: Record<ProviderType, string> = {
  ollama: "qwen3:8b",
  openai: "gpt-4o-mini",
  google: "gemini-2.5-flash"
};

export default function EmployeesPage() {
  const [sessionToken, setSessionToken] = useState("");
  const [agents, setAgents] = useState<AgentItem[]>([]);
  const [skills, setSkills] = useState<Array<{ id: string; name: string; enabled: number }>>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const [name, setName] = useState("");
  const [provider, setProvider] = useState<ProviderType>("ollama");
  const [model, setModel] = useState(providerModels.ollama);
  const [systemPrompt, setSystemPrompt] = useState("You are a helpful digital employee.");
  const [selectedSkills, setSelectedSkills] = useState<string[]>([]);

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
    Promise.all([fetchAgents(sessionToken), fetchSkills(sessionToken)])
      .then(([agentRows, skillRows]) => {
        setAgents(agentRows);
        setSkills(skillRows);
      })
      .catch((loadError) => {
        setError(loadError instanceof Error ? loadError.message : "Failed to load employees");
      })
      .finally(() => setLoading(false));
  }, [sessionToken]);

  const enabledSkills = useMemo(
    () => skills.filter((skill) => skill.enabled === 1),
    [skills]
  );

  async function onCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!name.trim()) {
      return;
    }

    setSaving(true);
    setError("");
    try {
      const created = await createAgent({
        sessionToken,
        name: name.trim(),
        provider,
        model: model.trim(),
        systemPrompt,
        skills: selectedSkills
      });
      setAgents((prev) => [created, ...prev]);
      setName("");
      setSystemPrompt("You are a helpful digital employee.");
      setSelectedSkills([]);
    } catch (createError) {
      setError(createError instanceof Error ? createError.message : "Failed to create employee");
    } finally {
      setSaving(false);
    }
  }

  async function onDelete(agentId: string) {
    try {
      await deleteAgent({ sessionToken, id: agentId });
      setAgents((prev) => prev.filter((agent) => agent.id !== agentId));
    } catch (deleteError) {
      setError(deleteError instanceof Error ? deleteError.message : "Failed to delete employee");
    }
  }

  return (
    <AppShell
      active="employees"
      title="Digital Employees"
      subtitle="Manage employee name, prompt, provider, model and bound skills."
    >
      <section className="grid gap-4 lg:grid-cols-[380px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Create Employee</CardTitle>
            <CardDescription>
              Configure one digital employee profile for channels and direct chat.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={onCreate} className="grid gap-3">
              <div className="grid gap-2">
                <Label htmlFor="employee-name">Name</Label>
                <Input
                  id="employee-name"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="Market Analyst"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="employee-provider">Provider</Label>
                <Select
                  id="employee-provider"
                  value={provider}
                  onChange={(event) => {
                    const next = event.target.value as ProviderType;
                    setProvider(next);
                    setModel(providerModels[next]);
                  }}
                  options={[
                    { value: "ollama", label: "Ollama" },
                    { value: "openai", label: "OpenAI" },
                    { value: "google", label: "Google Gemini" }
                  ]}
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="employee-model">Model</Label>
                <Input
                  id="employee-model"
                  value={model}
                  onChange={(event) => setModel(event.target.value)}
                  placeholder="qwen3:8b"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="employee-prompt">System Prompt</Label>
                <Textarea
                  id="employee-prompt"
                  value={systemPrompt}
                  onChange={(event) => setSystemPrompt(event.target.value)}
                  rows={5}
                />
              </div>

              <div className="grid gap-2">
                <Label>Skills</Label>
                <div className="max-h-40 space-y-2 overflow-y-auto rounded-lg border border-slate-200 p-2">
                  {enabledSkills.map((skill) => (
                    <label key={skill.id} className="flex items-center gap-2 text-sm text-slate-700">
                      <input
                        type="checkbox"
                        checked={selectedSkills.includes(skill.id)}
                        onChange={(event) => {
                          setSelectedSkills((prev) =>
                            event.target.checked
                              ? [...prev, skill.id]
                              : prev.filter((id) => id !== skill.id)
                          );
                        }}
                      />
                      {skill.name} ({skill.id})
                    </label>
                  ))}
                </div>
              </div>

              <Button type="submit" disabled={saving || !sessionToken}>
                {saving ? <Spinner className="h-4 w-4" /> : null}
                {saving ? "Creating..." : "Create Employee"}
              </Button>
            </form>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Employee List</CardTitle>
            <CardDescription>
              Active digital employees available in channels.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {loading ? (
              <p className="text-sm text-slate-500">Loading employees...</p>
            ) : agents.length === 0 ? (
              <p className="text-sm text-slate-500">No employee yet.</p>
            ) : (
              agents.map((agent) => (
                <div key={agent.id} className="rounded-lg border border-slate-200 p-3">
                  <div className="flex items-start justify-between gap-2">
                    <div>
                      <p className="text-sm font-semibold text-slate-900">{agent.name}</p>
                      <p className="text-xs text-slate-500">
                        {agent.provider} · {agent.model}
                      </p>
                    </div>
                    <Button variant="outline" size="sm" onClick={() => onDelete(agent.id)}>
                      Delete
                    </Button>
                  </div>
                  <p className="mt-2 whitespace-pre-wrap text-xs text-slate-600">
                    {agent.system_prompt || "(no prompt)"}
                  </p>
                  <div className="mt-2 flex flex-wrap gap-1">
                    {(agent.skills ?? []).map((skillId) => (
                      <Badge key={skillId} variant="secondary">
                        {skillId}
                      </Badge>
                    ))}
                  </div>
                </div>
              ))
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
