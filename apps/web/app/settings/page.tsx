"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { useRouter } from "next/navigation";
import { getOllamaSettings, updateOllamaSettings } from "../../lib/api";

const SESSION_KEY = "fae_session_token";

export default function SettingsPage() {
  const router = useRouter();
  const [sessionToken, setSessionToken] = useState<string>("");
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:11434");
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<string>("");
  const [error, setError] = useState<string>("");

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
    getOllamaSettings(sessionToken)
      .then((result) => {
        if (result?.ok && result?.data?.baseUrl) {
          setBaseUrl(String(result.data.baseUrl));
        }
      })
      .catch(() => {
        setError("Failed to load settings");
      });
  }, [sessionToken]);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setMessage("");
    setError("");
    setLoading(true);
    try {
      const result = await updateOllamaSettings({
        sessionToken,
        baseUrl
      });
      if (!result?.ok) {
        setError(result?.error?.message ?? "Failed to save settings");
        return;
      }
      setMessage("Settings saved");
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
    <main>
      <h1>Settings</h1>
      <p>Configure your local Ollama connection.</p>

      <p>
        <Link href="/chat">Back to chat</Link>
      </p>

      <form onSubmit={onSubmit} style={{ display: "grid", gap: 12, maxWidth: 640 }}>
        <label>
          Ollama Base URL
          <input
            value={baseUrl}
            onChange={(event) => setBaseUrl(event.target.value)}
            placeholder="http://127.0.0.1:11434"
            style={{ width: "100%", marginTop: 8 }}
          />
        </label>
        <button type="submit" disabled={loading || !sessionToken}>
          {loading ? "Saving..." : "Save"}
        </button>
      </form>

      {message ? <p style={{ color: "green" }}>{message}</p> : null}
      {error ? <p style={{ color: "tomato" }}>{error}</p> : null}
    </main>
  );
}
