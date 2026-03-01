"use client";

import { useState } from "react";
import type { FormEvent } from "react";
import { useRouter } from "next/navigation";
import { loginWithStartupToken } from "../../lib/api";

const SESSION_KEY = "fae_session_token";

export default function LoginPage() {
  const router = useRouter();
  const [token, setToken] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setLoading(true);
    try {
      const result = await loginWithStartupToken(token);
      if (!result.ok || !result.data?.sessionToken) {
        setError(result.error?.message ?? "Login failed");
        return;
      }
      localStorage.setItem(SESSION_KEY, result.data.sessionToken);
      router.replace("/chat");
    } catch (requestError) {
      setError(
        requestError instanceof Error ? requestError.message : "Login request failed"
      );
    } finally {
      setLoading(false);
    }
  }

  return (
    <main>
      <h1>fae Login</h1>
      <p>Paste your startup token from ~/.fae/startup-token</p>
      <form onSubmit={onSubmit} style={{ display: "grid", gap: 12, maxWidth: 520 }}>
        <input
          value={token}
          onChange={(event) => setToken(event.target.value)}
          placeholder="Startup token"
          autoComplete="off"
        />
        <button type="submit" disabled={loading}>
          {loading ? "Signing in..." : "Sign in"}
        </button>
      </form>
      {error ? <p style={{ color: "tomato" }}>{error}</p> : null}
    </main>
  );
}
