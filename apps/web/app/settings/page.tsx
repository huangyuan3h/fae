"use client";

import type { FormEvent } from "react";
import { useEffect, useState } from "react";
import { CircleCheck, LinkIcon } from "lucide-react";
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
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Spinner } from "../../components/ui/spinner";
import { getOllamaSettings, updateOllamaSettings } from "../../lib/api";
import { getSessionToken } from "../../lib/session";

export default function SettingsPage() {
  const router = useRouter();
  const [sessionToken, setSessionToken] = useState<string>("");
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:11434");
  const [loading, setLoading] = useState(false);
  const [initializing, setInitializing] = useState(true);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

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
    async function loadSettings() {
      setInitializing(true);
      setError("");
      try {
        const result = await getOllamaSettings(sessionToken);
        if (active) {
          setBaseUrl(result.baseUrl || "http://127.0.0.1:11434");
        }
      } catch (requestError) {
        if (active) {
          setError(
            requestError instanceof Error
              ? requestError.message
              : "Failed to load settings"
          );
        }
      } finally {
        if (active) {
          setInitializing(false);
        }
      }
    }

    void loadSettings();

    return () => {
      active = false;
    };
  }, [sessionToken]);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setMessage("");
    setError("");

    if (!sessionToken) {
      setError("Missing session token. Please sign in again.");
      return;
    }

    setLoading(true);
    try {
      const result = await updateOllamaSettings({
        sessionToken,
        baseUrl: baseUrl.trim()
      });
      setBaseUrl(result.baseUrl);
      setMessage("Settings saved.");
    } catch (requestError) {
      setError(
        requestError instanceof Error
          ? requestError.message
          : "Failed to save settings"
      );
    } finally {
      setLoading(false);
    }
  }

  return (
    <AppShell
      active="settings"
      title="Runtime Settings"
      subtitle="Configure how the daemon connects to local model providers."
    >
      <section className="grid gap-4 lg:grid-cols-[1fr_320px]">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Ollama</CardTitle>
            <CardDescription>
              Update the base endpoint used by the daemon for chat streaming.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={onSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="ollama-url">Ollama Base URL</Label>
                <div className="relative">
                  <LinkIcon className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                  <Input
                    id="ollama-url"
                    value={baseUrl}
                    onChange={(event) => setBaseUrl(event.target.value)}
                    placeholder="http://127.0.0.1:11434"
                    className="pl-9"
                    disabled={initializing}
                  />
                </div>
              </div>

              <Button type="submit" disabled={loading || initializing || !sessionToken}>
                {loading ? (
                  <>
                    <Spinner className="h-4 w-4" />
                    Saving...
                  </>
                ) : (
                  "Save"
                )}
              </Button>
            </form>

            {message ? (
              <Alert variant="success" className="mt-4">
                <AlertTitle className="inline-flex items-center gap-2">
                  <CircleCheck className="h-4 w-4" />
                  Saved
                </AlertTitle>
                <AlertDescription>{message}</AlertDescription>
              </Alert>
            ) : null}

            {error ? (
              <Alert variant="destructive" className="mt-4">
                <AlertTitle>Save failed</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Connection Status</CardTitle>
            <CardDescription>Current frontend state.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Badge variant={initializing ? "secondary" : "default"}>
              {initializing ? "Loading config..." : "Config loaded"}
            </Badge>
            <p className="rounded-xl border border-slate-700 bg-slate-900/60 p-3 font-[family-name:var(--font-ibm-plex-mono)] text-xs text-slate-300">
              {baseUrl}
            </p>
            <p className="text-xs text-slate-500">
              Use a full URL including protocol, such as
              <span className="font-[family-name:var(--font-ibm-plex-mono)] text-slate-300">
                {" "}http://127.0.0.1:11434
              </span>
              .
            </p>
          </CardContent>
        </Card>
      </section>
    </AppShell>
  );
}
