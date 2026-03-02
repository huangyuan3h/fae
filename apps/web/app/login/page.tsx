"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { createDevSession } from "../../lib/api";
import { ensureSessionToken } from "../../lib/session";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui/card";
import { Spinner } from "../../components/ui/spinner";

export default function LoginPage() {
  const router = useRouter();
  useEffect(() => {
    ensureSessionToken(createDevSession)
      .then(() => {
        router.replace("/chat");
      })
      .catch(() => {
        router.replace("/chat");
      });
  }, [router]);

  return (
    <main className="mx-auto flex min-h-screen w-full max-w-3xl items-center px-4 py-8 sm:px-6">
      <Card className="w-full">
        <CardHeader className="space-y-3">
          <CardTitle className="text-3xl">Preparing workspace</CardTitle>
          <CardDescription>Login is disabled in development. Redirecting to chat...</CardDescription>
        </CardHeader>
        <CardContent className="flex items-center gap-3 text-slate-600">
          <Spinner className="h-4 w-4" />
          Redirecting...
        </CardContent>
      </Card>
    </main>
  );
}
