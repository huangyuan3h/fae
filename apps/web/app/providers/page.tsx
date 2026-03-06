"use client";

import type { FormEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import { Pencil, Plus, Trash2, X } from "lucide-react";
import { AppShell } from "../../components/app-shell";
import { Alert, AlertDescription, AlertTitle } from "../../components/ui/alert";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui/card";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Select } from "../../components/ui/select";
import { Spinner } from "../../components/ui/spinner";
import {
  createDevSession,
  getProviderSettings,
  type ProviderConfig,
  type ProviderSettings,
  type ProviderType,
  updateProviderSettings
} from "../../lib/api";
import { ensureSessionToken } from "../../lib/session";

const defaultBaseUrl: Record<ProviderType, string> = {
  ollama: "http://127.0.0.1:11434",
  openai: "",
  google: "",
  alibaba: "https://dashscope.aliyuncs.com/compatible-mode/v1"
};

const defaultModelByType: Record<ProviderType, string> = {
  ollama: "",
  openai: "",
  google: "",
  alibaba: "qwen-turbo"
};

const providerDescriptions: Record<ProviderType, string> = {
  ollama: "Run local models through Ollama with your local base URL.",
  openai: "Use OpenAI-hosted models with one or more API keys.",
  google: "Use Google Gemini models with one or more API keys.",
  alibaba: "Use Alibaba Bailian (DashScope) models with API keys."
};

const emptySettings: ProviderSettings = {
  providerConfigs: []
};

function providerLabel(type: ProviderType): string {
  if (type === "openai") {
    return "OpenAI";
  }
  if (type === "google") {
    return "Google Gemini";
  }
  if (type === "alibaba") {
    return "Alibaba Bailian";
  }
  return "Ollama";
}

export default function ProvidersPage() {
  const [sessionToken, setSessionToken] = useState("");
  const [settings, setSettings] = useState<ProviderSettings>(emptySettings);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingConfigId, setEditingConfigId] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  const [name, setName] = useState("");
  const [type, setType] = useState<ProviderType>("ollama");
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState(defaultBaseUrl.ollama);
  const [modelId, setModelId] = useState(defaultModelByType.ollama);

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
    getProviderSettings(sessionToken)
      .then((loaded) => {
        setSettings(loaded);
      })
      .catch((loadError) => {
        setError(loadError instanceof Error ? loadError.message : "Failed to load providers");
      })
      .finally(() => {
        setLoading(false);
      });
  }, [sessionToken]);

  const providerCards = useMemo(
    () => settings.providerConfigs,
    [settings.providerConfigs]
  );

  function resetForm() {
    setEditingConfigId(null);
    setName("");
    setType("ollama");
    setApiKey("");
    setBaseUrl(defaultBaseUrl.ollama);
    setModelId(defaultModelByType.ollama);
  }

  function openCreateModal() {
    setError("");
    setMessage("");
    resetForm();
    setModalOpen(true);
  }

  function openEditModal(config: ProviderConfig) {
    setError("");
    setMessage("");
    setEditingConfigId(config.id);
    setName(config.name);
    setType(config.type);
    setApiKey(config.apiKey);
    setBaseUrl(config.baseUrl);
    setModelId(config.modelId ?? defaultModelByType[config.type]);
    setModalOpen(true);
  }

  async function saveSettings(nextConfigs: ProviderConfig[]) {
    setSaving(true);
    try {
      const saved = await updateProviderSettings({
        sessionToken,
        settings: {
          providerConfigs: nextConfigs
        }
      });
      setSettings(saved);
      setMessage("Provider settings saved.");
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Failed to save providers");
    } finally {
      setSaving(false);
    }
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!name.trim()) {
      return;
    }

    setError("");
    setMessage("");

    const normalizedBaseUrl = baseUrl.trim() || defaultBaseUrl[type];
    const configPayload: ProviderConfig = {
      id: editingConfigId ?? crypto.randomUUID(),
      name: name.trim(),
      type,
      apiKey: type === "ollama" ? "" : apiKey.trim(),
      baseUrl: normalizedBaseUrl,
      modelId: modelId.trim(),
      enabled: true
    };

    const nextConfigs = editingConfigId
      ? settings.providerConfigs.map((config) =>
          config.id === editingConfigId ? configPayload : config
        )
      : [configPayload, ...settings.providerConfigs];

    await saveSettings(nextConfigs);
    setModalOpen(false);
    resetForm();
  }

  async function onDelete(configId: string) {
    setError("");
    setMessage("");
    const nextConfigs = settings.providerConfigs.filter((config) => config.id !== configId);
    await saveSettings(nextConfigs);
  }

  return (
    <AppShell active="providers" title="Provider Settings">
      <section className="space-y-4">
        <div className="flex items-center justify-end">
          <Button type="button" onClick={openCreateModal} disabled={!sessionToken || saving}>
            <Plus className="h-4 w-4" />
            Add Provider
          </Button>
        </div>

        {loading ? (
          <Card>
            <CardContent className="py-6">
              <p className="text-sm text-slate-500">Loading providers...</p>
            </CardContent>
          </Card>
        ) : providerCards.length === 0 ? (
          <Card>
            <CardContent className="py-6">
              <p className="text-sm text-slate-500">No provider configured yet.</p>
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-4 lg:grid-cols-2">
            {providerCards.map((config) => (
              <Card key={config.id} className="min-h-[220px] border-slate-200">
                <CardHeader>
                  <div className="flex items-start justify-between gap-3">
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <CardTitle className="text-base">{config.name}</CardTitle>
                        <Badge variant="secondary">{providerLabel(config.type)}</Badge>
                      </div>
                      <CardDescription>{providerDescriptions[config.type]}</CardDescription>
                    </div>

                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        onClick={() => openEditModal(config)}
                        className="rounded-md p-1.5 text-slate-500 transition hover:bg-slate-100 hover:text-slate-700"
                        aria-label={`Edit ${config.name}`}
                      >
                        <Pencil className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onDelete(config.id)}
                        className="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-50 hover:text-rose-600"
                        aria-label={`Delete ${config.name}`}
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                </CardHeader>

                <CardContent className="space-y-4">
                  <div className="space-y-1">
                    <p className="text-xs font-medium uppercase tracking-wide text-slate-500">Base URL</p>
                    <p className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                      {config.baseUrl}
                    </p>
                  </div>

                  {config.type !== "ollama" ? (
                    <div className="space-y-1">
                      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">API Key</p>
                      <p className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                        {config.apiKey ? `${config.apiKey.slice(0, 4)}••••••••` : "(empty)"}
                      </p>
                    </div>
                  ) : null}

                  {config.type === "ollama" || config.type === "alibaba" ? (
                    <div className="space-y-1">
                      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">Model ID</p>
                      <p className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                        {config.modelId || defaultModelByType[config.type]}
                      </p>
                    </div>
                  ) : null}
                </CardContent>
              </Card>
            ))}
          </div>
        )}

        {message ? (
          <Alert variant="success">
            <AlertTitle>Saved</AlertTitle>
            <AlertDescription>{message}</AlertDescription>
          </Alert>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
      </section>

      {modalOpen ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 p-4">
          <div className="w-full max-w-xl rounded-xl border border-slate-200 bg-white shadow-xl">
            <div className="flex items-center justify-between border-b border-slate-200 px-5 py-4">
              <div>
                <h2 className="text-base font-semibold text-slate-900">
                  {editingConfigId ? "Edit Provider" : "Add Provider"}
                </h2>
                <p className="text-sm text-slate-500">
                  Add named provider configurations. Same type can be added multiple times.
                </p>
              </div>
              <button
                type="button"
                className="rounded-lg p-1 text-slate-500 transition hover:bg-slate-100"
                onClick={() => {
                  setModalOpen(false);
                  resetForm();
                }}
                aria-label="Close provider modal"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <form onSubmit={onSubmit} className="grid gap-4 px-5 py-4">
              <div className="grid gap-2">
                <Label htmlFor="provider-name">Name</Label>
                <Input
                  id="provider-name"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="OpenAI - Finance Team"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="provider-type">Type</Label>
                <Select
                  id="provider-type"
                  value={type}
                  onChange={(event) => {
                    const nextType = event.target.value as ProviderType;
                    setType(nextType);
                    setBaseUrl(defaultBaseUrl[nextType]);
                    if (nextType === "ollama") {
                      setModelId((current) =>
                        current.trim() ? current : defaultModelByType[nextType]
                      );
                      setApiKey("");
                    } else if (nextType === "alibaba") {
                      setModelId((current) =>
                        current.trim() ? current : defaultModelByType[nextType]
                      );
                    } else {
                      // openai, google: clear modelId but allow user to set it
                      setModelId(defaultModelByType[nextType]);
                    }
                  }}
                  options={[
                    { value: "ollama", label: "Ollama" },
                    { value: "openai", label: "OpenAI" },
                    { value: "google", label: "Google Gemini" },
                    { value: "alibaba", label: "Alibaba Bailian" }
                  ]}
                />
              </div>

              {type !== "ollama" ? (
                <div className="grid gap-2">
                  <Label htmlFor="provider-api-key">API Key</Label>
                  <Input
                    id="provider-api-key"
                    type="password"
                    value={apiKey}
                    onChange={(event) => setApiKey(event.target.value)}
                    placeholder="Required for cloud providers"
                  />
                </div>
              ) : null}

              <div className="grid gap-2">
                <Label htmlFor="provider-base-url">
                  {type === "ollama" ? "Base URL" : "Base URL (Optional)"}
                </Label>
                <Input
                  id="provider-base-url"
                  value={baseUrl}
                  onChange={(event) => setBaseUrl(event.target.value)}
                  placeholder={defaultBaseUrl[type]}
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="provider-model-id">Model ID</Label>
                <Input
                  id="provider-model-id"
                  value={modelId}
                  onChange={(event) => setModelId(event.target.value)}
                  placeholder={
                    type === "ollama" ? "qwen3:8b" :
                    type === "openai" ? "gpt-4o-mini" :
                    type === "google" ? "gemini-2.5-flash" :
                    "qwen-turbo"
                  }
                />
              </div>

              <div className="flex items-center justify-end gap-2 border-t border-slate-200 pt-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    setModalOpen(false);
                    resetForm();
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
                    (type !== "ollama" && !apiKey.trim()) ||
                    (type === "ollama" && !baseUrl.trim()) ||
                    !modelId.trim()
                  }
                >
                  {saving ? <Spinner className="h-4 w-4" /> : null}
                  {saving ? "Saving..." : editingConfigId ? "Save Changes" : "Add Provider"}
                </Button>
              </div>
            </form>
          </div>
        </div>
      ) : null}
    </AppShell>
  );
}
