"use client";

import type { FormEvent } from "react";
import { useEffect, useState } from "react";
import { AppShell } from "../../components/app-shell";
import { Alert, AlertDescription, AlertTitle } from "../../components/ui/alert";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui/card";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Select } from "../../components/ui/select";
import { Spinner } from "../../components/ui/spinner";
import {
  createDevSession,
  getProviderSettings,
  type ProviderSettings,
  type ProviderType,
  updateProviderSettings
} from "../../lib/api";
import { ensureSessionToken } from "../../lib/session";

const emptySettings: ProviderSettings = {
  defaultProvider: "ollama",
  ollama: { baseUrl: "http://127.0.0.1:11434" },
  openai: { apiKey: "", baseUrl: "https://api.openai.com/v1" },
  google: {
    apiKey: "",
    baseUrl: "https://generativelanguage.googleapis.com/v1beta"
  }
};

export default function ProvidersPage() {
  const [sessionToken, setSessionToken] = useState("");
  const [settings, setSettings] = useState<ProviderSettings>(emptySettings);
  const [initializing, setInitializing] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    ensureSessionToken(createDevSession)
      .then((token) => setSessionToken(token))
      .catch(() => setError("Failed to create development session."));
  }, []);

  useEffect(() => {
    if (!sessionToken) {
      return;
    }

    setInitializing(true);
    getProviderSettings(sessionToken)
      .then((loaded) => {
        setSettings(loaded);
      })
      .catch((loadError) => {
        setError(loadError instanceof Error ? loadError.message : "Failed to load settings");
      })
      .finally(() => {
        setInitializing(false);
      });
  }, [sessionToken]);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError("");
    setMessage("");
    setSaving(true);

    try {
      const saved = await updateProviderSettings({
        sessionToken,
        settings
      });
      setSettings(saved);
      setMessage("Provider settings saved.");
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Failed to save providers");
    } finally {
      setSaving(false);
    }
  }

  return (
    <AppShell
      active="providers"
      title="Provider Settings"
      subtitle="Configure Ollama, OpenAI, and Gemini connections for all digital employees."
    >
      <Card>
        <CardHeader>
          <CardTitle>Model Providers</CardTitle>
          <CardDescription>
            API keys are stored locally on your machine in the daemon settings table.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="default-provider">Default provider</Label>
              <Select
                id="default-provider"
                value={settings.defaultProvider}
                onChange={(event) =>
                  setSettings((prev) => ({
                    ...prev,
                    defaultProvider: event.target.value as ProviderType
                  }))
                }
                disabled={initializing}
                options={[
                  { value: "ollama", label: "Ollama" },
                  { value: "openai", label: "OpenAI" },
                  { value: "google", label: "Google Gemini" }
                ]}
              />
            </div>

            <div className="grid gap-2 rounded-lg border border-slate-200 bg-white p-3">
              <Label htmlFor="ollama-base">Ollama base URL</Label>
              <Input
                id="ollama-base"
                value={settings.ollama.baseUrl}
                onChange={(event) =>
                  setSettings((prev) => ({
                    ...prev,
                    ollama: { ...prev.ollama, baseUrl: event.target.value }
                  }))
                }
                placeholder="http://127.0.0.1:11434"
              />
            </div>

            <div className="grid gap-2 rounded-lg border border-slate-200 bg-white p-3">
              <Label htmlFor="openai-key">OpenAI API Key</Label>
              <Input
                id="openai-key"
                type="password"
                value={settings.openai.apiKey}
                onChange={(event) =>
                  setSettings((prev) => ({
                    ...prev,
                    openai: { ...prev.openai, apiKey: event.target.value }
                  }))
                }
                placeholder="sk-..."
              />
              <Label htmlFor="openai-base">OpenAI base URL</Label>
              <Input
                id="openai-base"
                value={settings.openai.baseUrl}
                onChange={(event) =>
                  setSettings((prev) => ({
                    ...prev,
                    openai: { ...prev.openai, baseUrl: event.target.value }
                  }))
                }
                placeholder="https://api.openai.com/v1"
              />
            </div>

            <div className="grid gap-2 rounded-lg border border-slate-200 bg-white p-3">
              <Label htmlFor="google-key">Google API Key</Label>
              <Input
                id="google-key"
                type="password"
                value={settings.google.apiKey}
                onChange={(event) =>
                  setSettings((prev) => ({
                    ...prev,
                    google: { ...prev.google, apiKey: event.target.value }
                  }))
                }
                placeholder="AIza..."
              />
              <Label htmlFor="google-base">Google base URL</Label>
              <Input
                id="google-base"
                value={settings.google.baseUrl}
                onChange={(event) =>
                  setSettings((prev) => ({
                    ...prev,
                    google: { ...prev.google, baseUrl: event.target.value }
                  }))
                }
                placeholder="https://generativelanguage.googleapis.com/v1beta"
              />
            </div>

            <div className="flex items-center gap-2">
              <Button type="submit" disabled={initializing || saving || !sessionToken}>
                {saving ? <Spinner className="h-4 w-4" /> : null}
                {saving ? "Saving..." : "Save Providers"}
              </Button>
            </div>
          </form>

          {message ? (
            <Alert variant="success" className="mt-4">
              <AlertTitle>Saved</AlertTitle>
              <AlertDescription>{message}</AlertDescription>
            </Alert>
          ) : null}

          {error ? (
            <Alert variant="destructive" className="mt-4">
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : null}
        </CardContent>
      </Card>
    </AppShell>
  );
}
