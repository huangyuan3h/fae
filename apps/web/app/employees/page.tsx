"use client";

import type { ChangeEvent, FormEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import { Pencil, Plus, Trash2, Upload, X } from "lucide-react";
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
  getProviderSettings,
  updateAgent,
  type AgentItem,
  type ProviderConfig,
  type ProviderType,
  type SkillItem
} from "../../lib/api";
import { ensureSessionToken } from "../../lib/session";

const providerModels: Record<ProviderType, string> = {
  ollama: "qwen3:8b",
  openai: "gpt-4o-mini",
  google: "gemini-2.5-flash"
};

const presetAvatars = ["🤖", "🧠", "📊", "🛰", "🛡", "🔥", "🧩", "🦉"];

function isImageAvatar(value: string): boolean {
  return value.startsWith("data:image/") || value.startsWith("http://") || value.startsWith("https://");
}

function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/).slice(0, 2);
  if (parts.length === 0 || !parts[0]) {
    return "FA";
  }
  return parts.map((part) => part[0]?.toUpperCase() ?? "").join("");
}

function AgentAvatar(props: { avatar: string | null | undefined; name: string }) {
  const avatar = props.avatar ?? "";

  if (avatar.startsWith("emoji:")) {
    return (
      <div className="inline-flex h-12 w-12 items-center justify-center rounded-full border border-blue-200 bg-blue-50 text-2xl">
        {avatar.slice(6)}
      </div>
    );
  }

  if (avatar && isImageAvatar(avatar)) {
    return (
      <img
        src={avatar}
        alt={`${props.name} avatar`}
        className="h-12 w-12 rounded-full border border-slate-200 object-cover"
      />
    );
  }

  return (
    <div className="inline-flex h-12 w-12 items-center justify-center rounded-full border border-blue-200 bg-blue-50 text-sm font-semibold text-blue-700">
      {getInitials(props.name)}
    </div>
  );
}

export default function EmployeesPage() {
  const [sessionToken, setSessionToken] = useState("");
  const [agents, setAgents] = useState<AgentItem[]>([]);
  const [skills, setSkills] = useState<SkillItem[]>([]);
  const [providerConfigs, setProviderConfigs] = useState<ProviderConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [editingAgentId, setEditingAgentId] = useState<string | null>(null);
  const [error, setError] = useState("");

  const [name, setName] = useState("");
  const [selectedProviderConfigId, setSelectedProviderConfigId] = useState("");
  const [provider, setProvider] = useState<ProviderType>("ollama");
  const [model, setModel] = useState(providerModels.ollama);
  const [systemPrompt, setSystemPrompt] = useState("You are a helpful digital employee.");
  const [selectedSkills, setSelectedSkills] = useState<string[]>([]);
  const [avatarMode, setAvatarMode] = useState<"preset" | "upload">("preset");
  const [selectedPresetAvatar, setSelectedPresetAvatar] = useState(presetAvatars[0]);
  const [uploadedAvatar, setUploadedAvatar] = useState<string | null>(null);

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
    Promise.all([
      fetchAgents(sessionToken),
      fetchSkills(sessionToken),
      getProviderSettings(sessionToken)
    ])
      .then(([agentRows, skillRows, providerSettings]) => {
        setAgents(agentRows);
        setSkills(skillRows);
        setProviderConfigs(providerSettings.providerConfigs);
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

  const configuredProviders = useMemo(
    () => providerConfigs.filter((config) => config.enabled),
    [providerConfigs]
  );

  const providerSelectOptions = useMemo(
    () =>
      configuredProviders.map((config) => ({
        value: config.id,
        label: `${config.name} (${config.type})`
      })),
    [configuredProviders]
  );

  useEffect(() => {
    if (configuredProviders.length === 0) {
      return;
    }
    if (!configuredProviders.some((config) => config.id === selectedProviderConfigId)) {
      const nextConfig = configuredProviders[0];
      if (!nextConfig) {
        return;
      }
      setSelectedProviderConfigId(nextConfig.id);
      setProvider(nextConfig.type);
      if (nextConfig.type === "ollama") {
        setModel(nextConfig.modelId?.trim() ?? "");
      } else {
        setModel(providerModels[nextConfig.type]);
      }
    }
  }, [configuredProviders, selectedProviderConfigId]);

  const currentAvatarValue = useMemo(() => {
    if (avatarMode === "upload" && uploadedAvatar) {
      return uploadedAvatar;
    }
    return `emoji:${selectedPresetAvatar}`;
  }, [avatarMode, selectedPresetAvatar, uploadedAvatar]);

  function resetCreateForm() {
    const defaultConfig = configuredProviders[0] ?? null;
    const defaultProvider = defaultConfig?.type ?? "ollama";
    setName("");
    setSelectedProviderConfigId(defaultConfig?.id ?? "");
    setProvider(defaultProvider);
    if (defaultConfig?.type === "ollama") {
      setModel(defaultConfig.modelId?.trim() ?? "");
    } else {
      setModel(providerModels[defaultProvider]);
    }
    setSystemPrompt("You are a helpful digital employee.");
    setSelectedSkills([]);
    setAvatarMode("preset");
    setSelectedPresetAvatar(presetAvatars[0]);
    setUploadedAvatar(null);
  }

  function openCreateModal() {
    setError("");
    setEditingAgentId(null);
    resetCreateForm();
    setCreateOpen(true);
  }

  function openEditModal(agent: AgentItem) {
    setError("");
    setEditingAgentId(agent.id);
    setName(agent.name);
    const matchedConfig =
      configuredProviders.find((config) => config.id === (agent.provider_config_id ?? "")) ??
      configuredProviders.find((config) => config.type === agent.provider) ??
      null;

    setSelectedProviderConfigId(matchedConfig?.id ?? "");
    setProvider(matchedConfig?.type ?? agent.provider);
    setModel(agent.model);
    setSystemPrompt(agent.system_prompt ?? "You are a helpful digital employee.");
    setSelectedSkills(agent.skills ?? []);

    const avatar = agent.avatar_url ?? "";
    if (avatar.startsWith("emoji:")) {
      const emojiValue = avatar.slice(6);
      setAvatarMode("preset");
      setSelectedPresetAvatar(
        presetAvatars.includes(emojiValue) ? emojiValue : presetAvatars[0]
      );
      setUploadedAvatar(null);
    } else if (avatar) {
      setAvatarMode("upload");
      setUploadedAvatar(avatar);
      setSelectedPresetAvatar(presetAvatars[0]);
    } else {
      setAvatarMode("preset");
      setSelectedPresetAvatar(presetAvatars[0]);
      setUploadedAvatar(null);
    }

    setCreateOpen(true);
  }

  async function onAvatarUpload(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }

    if (!file.type.startsWith("image/")) {
      setError("Avatar file must be an image.");
      return;
    }

    if (file.size > 1024 * 1024) {
      setError("Avatar image must be <= 1MB.");
      return;
    }

    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === "string") {
        setUploadedAvatar(reader.result);
      }
    };
    reader.readAsDataURL(file);
  }

  async function onSubmitAgent(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!name.trim()) {
      return;
    }
    const selectedConfig = configuredProviders.find(
      (config) => config.id === selectedProviderConfigId
    );
    if (!selectedConfig) {
      setError("Please configure a provider first.");
      return;
    }

    setSaving(true);
    setError("");

    try {
      if (editingAgentId) {
        const updated = await updateAgent({
          sessionToken,
          id: editingAgentId,
          name: name.trim(),
          provider: selectedConfig.type,
          providerConfigId: selectedConfig.id,
          model:
            model.trim() ||
            (selectedConfig.type === "ollama"
              ? selectedConfig.modelId?.trim() || "qwen3:8b"
              : providerModels[selectedConfig.type]),
          systemPrompt,
          avatarUrl: currentAvatarValue,
          skills: selectedSkills
        });
        setAgents((prev) => prev.map((agent) => (agent.id === updated.id ? updated : agent)));
      } else {
        const created = await createAgent({
          sessionToken,
          name: name.trim(),
          provider: selectedConfig.type,
          providerConfigId: selectedConfig.id,
          model:
            model.trim() ||
            (selectedConfig.type === "ollama"
              ? selectedConfig.modelId?.trim() || "qwen3:8b"
              : providerModels[selectedConfig.type]),
          systemPrompt,
          avatarUrl: currentAvatarValue,
          skills: selectedSkills
        });
        setAgents((prev) => [created, ...prev]);
      }
      setCreateOpen(false);
      setEditingAgentId(null);
      resetCreateForm();
    } catch (createError) {
      setError(createError instanceof Error ? createError.message : "Failed to save employee");
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
    >
      <section className="space-y-4">
        <div className="flex items-center justify-end">
          <Button
            type="button"
            onClick={openCreateModal}
          >
            <Plus className="h-4 w-4" />
            Create Employee
          </Button>
        </div>

        {loading ? (
          <Card>
            <CardContent className="py-6">
              <p className="text-sm text-slate-500">Loading employees...</p>
            </CardContent>
          </Card>
        ) : agents.length === 0 ? (
          <Card>
            <CardContent className="py-6">
              <p className="text-sm text-slate-500">No employee yet. Create your first one.</p>
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {agents.map((agent) => (
              <Card key={agent.id} className="border-slate-200">
                <CardHeader className="space-y-3">
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex items-center gap-3">
                      <AgentAvatar avatar={agent.avatar_url} name={agent.name} />
                      <div>
                        <CardTitle className="text-base">{agent.name}</CardTitle>
                        <CardDescription>
                          {agent.provider} · {agent.model}
                        </CardDescription>
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        onClick={() => openEditModal(agent)}
                        className="rounded-md p-1.5 text-slate-500 transition hover:bg-slate-100 hover:text-slate-700"
                        aria-label={`Edit ${agent.name}`}
                      >
                        <Pencil className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onDelete(agent.id)}
                        className="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-50 hover:text-rose-600"
                        aria-label={`Delete ${agent.name}`}
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <p className="max-h-24 overflow-hidden whitespace-pre-wrap text-xs text-slate-600">
                    {agent.system_prompt || "(no prompt)"}
                  </p>
                  <div className="mt-3 flex flex-wrap gap-1">
                    {(agent.skills ?? []).length === 0 ? (
                      <Badge variant="secondary">No skills</Badge>
                    ) : (
                      (agent.skills ?? []).map((skillId) => (
                        <Badge key={skillId} variant="secondary">
                          {skillId}
                        </Badge>
                      ))
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}

        {error ? (
          <Alert variant="destructive">
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
      </section>

      {createOpen ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 p-4">
          <div className="max-h-[92vh] w-full max-w-2xl overflow-y-auto rounded-xl border border-slate-200 bg-white shadow-xl">
            <div className="flex items-center justify-between border-b border-slate-200 px-5 py-4">
              <div>
                <h2 className="text-base font-semibold text-slate-900">
                  {editingAgentId ? "Edit Employee" : "Create Employee"}
                </h2>
                <p className="text-sm text-slate-500">
                  {editingAgentId
                    ? "Update profile, model, skills and avatar."
                    : "Configure one digital employee profile for channels and direct chat."}
                </p>
              </div>
              <button
                type="button"
                className="rounded-lg p-1 text-slate-500 transition hover:bg-slate-100"
                onClick={() => {
                  setCreateOpen(false);
                  setEditingAgentId(null);
                  resetCreateForm();
                }}
                aria-label="Close create employee modal"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <form onSubmit={onSubmitAgent} className="grid gap-4 px-5 py-4">
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
                <Label>Avatar</Label>

                <div className="flex flex-wrap gap-2">
                  <Button
                    type="button"
                    variant={avatarMode === "preset" ? "default" : "outline"}
                    size="sm"
                    onClick={() => setAvatarMode("preset")}
                  >
                    Preset
                  </Button>
                  <Button
                    type="button"
                    variant={avatarMode === "upload" ? "default" : "outline"}
                    size="sm"
                    onClick={() => setAvatarMode("upload")}
                  >
                    Upload
                  </Button>
                </div>

                {avatarMode === "preset" ? (
                  <div className="grid grid-cols-8 gap-2 rounded-lg border border-slate-200 p-2">
                    {presetAvatars.map((avatar) => (
                      <button
                        key={avatar}
                        type="button"
                        onClick={() => setSelectedPresetAvatar(avatar)}
                        className={
                          selectedPresetAvatar === avatar
                            ? "inline-flex h-10 w-10 items-center justify-center rounded-lg border border-blue-400 bg-blue-50 text-xl"
                            : "inline-flex h-10 w-10 items-center justify-center rounded-lg border border-slate-200 text-xl hover:border-blue-300"
                        }
                        aria-label={`Choose avatar ${avatar}`}
                      >
                        {avatar}
                      </button>
                    ))}
                  </div>
                ) : (
                  <div className="grid gap-2 rounded-lg border border-slate-200 p-3">
                    <Label htmlFor="employee-avatar-upload" className="text-xs text-slate-500">
                      Image max 1MB
                    </Label>
                    <Input
                      id="employee-avatar-upload"
                      type="file"
                      accept="image/*"
                      onChange={onAvatarUpload}
                    />
                    {uploadedAvatar ? (
                      <img
                        src={uploadedAvatar}
                        alt="Uploaded avatar preview"
                        className="h-16 w-16 rounded-full border border-slate-200 object-cover"
                      />
                    ) : null}
                    <p className="text-xs text-slate-500">PNG/JPG/WebP all supported.</p>
                  </div>
                )}
              </div>

              <div className="grid gap-2 sm:grid-cols-2">
                <div className="grid gap-2">
                  <Label htmlFor="employee-provider">Provider</Label>
                  <Select
                    id="employee-provider"
                    value={selectedProviderConfigId}
                    onChange={(event) => {
                      const nextId = event.target.value;
                      setSelectedProviderConfigId(nextId);
                      const nextConfig = configuredProviders.find((config) => config.id === nextId);
                      if (nextConfig) {
                        setProvider(nextConfig.type);
                        if (nextConfig.type === "ollama") {
                          setModel(nextConfig.modelId?.trim() ?? "");
                        } else {
                          setModel(providerModels[nextConfig.type]);
                        }
                      }
                    }}
                    options={
                      providerSelectOptions.length > 0
                        ? providerSelectOptions
                        : [{ value: "", label: "No configured provider" }]
                    }
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

              <div className="flex items-center justify-end gap-2 border-t border-slate-200 pt-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    setCreateOpen(false);
                    setEditingAgentId(null);
                    resetCreateForm();
                  }}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={
                    saving ||
                    !sessionToken ||
                    !name.trim() ||
                    providerSelectOptions.length === 0 ||
                    !selectedProviderConfigId
                  }
                >
                  {saving ? <Spinner className="h-4 w-4" /> : <Upload className="h-4 w-4" />}
                  {saving ? "Saving..." : editingAgentId ? "Save Changes" : "Create Employee"}
                </Button>
              </div>
            </form>
          </div>
        </div>
      ) : null}
    </AppShell>
  );
}
