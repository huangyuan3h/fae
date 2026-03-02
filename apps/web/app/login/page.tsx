"use client";

import type { FormEvent } from "react";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { KeyRound, ShieldCheck } from "lucide-react";
import { loginWithStartupToken } from "../../lib/api";
import { saveSessionToken } from "../../lib/session";
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
import { Alert, AlertDescription, AlertTitle } from "../../components/ui/alert";
import { Spinner } from "../../components/ui/spinner";

export default function LoginPage() {
  const router = useRouter();
  const [token, setToken] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);

    if (!token.trim()) {
      setError("Startup token is required.");
      return;
    }

    setLoading(true);
    try {
      const sessionToken = await loginWithStartupToken(token.trim());
      saveSessionToken(sessionToken);
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
    <main className="mx-auto flex min-h-screen w-full max-w-3xl items-center px-4 py-8 sm:px-6">
      <Card className="w-full border-slate-700/70 bg-slate-950/70">
        <CardHeader className="space-y-3">
          <p className="inline-flex w-fit items-center gap-2 rounded-full border border-emerald-300/25 bg-emerald-400/10 px-3 py-1 text-xs font-medium uppercase tracking-[0.16em] text-emerald-200">
            <ShieldCheck className="h-3.5 w-3.5" />
            Local Auth
          </p>
          <CardTitle className="text-3xl">Sign in to fae</CardTitle>
          <CardDescription>
            Paste the startup token from <code className="font-[family-name:var(--font-ibm-plex-mono)] text-slate-300">~/.fae/startup-token</code> to unlock this local workspace.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="startup-token">Startup token</Label>
              <div className="relative">
                <KeyRound className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <Input
                  id="startup-token"
                  value={token}
                  onChange={(event) => setToken(event.target.value)}
                  placeholder="Paste token"
                  autoComplete="off"
                  className="pl-9"
                />
              </div>
            </div>

            <Button type="submit" disabled={loading} className="w-full">
              {loading ? (
                <>
                  <Spinner className="h-4 w-4" />
                  Signing in...
                </>
              ) : (
                "Sign in"
              )}
            </Button>
          </form>

          {error ? (
            <Alert variant="destructive" className="mt-4">
              <AlertTitle>Login failed</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : null}
        </CardContent>
      </Card>
    </main>
  );
}
