"use client";

import { useEffect, useState } from "react";
import { AppShell } from "../../components/app-shell";
import { Alert, AlertDescription, AlertTitle } from "../../components/ui/alert";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui/card";
import { Spinner } from "../../components/ui/spinner";
import {
  createDevSession,
  fetchSkills,
  type SkillItem,
  updateSkill
} from "../../lib/api";
import { ensureSessionToken } from "../../lib/session";

export default function SkillsPage() {
  const [sessionToken, setSessionToken] = useState("");
  const [skills, setSkills] = useState<SkillItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [pendingId, setPendingId] = useState<string | null>(null);
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

    setLoading(true);
    fetchSkills(sessionToken)
      .then((rows) => setSkills(rows))
      .catch((loadError) => {
        setError(loadError instanceof Error ? loadError.message : "Failed to load skills");
      })
      .finally(() => setLoading(false));
  }, [sessionToken]);

  async function onToggle(skill: SkillItem) {
    if (!sessionToken) {
      return;
    }

    setError("");
    setPendingId(skill.id);

    try {
      const updated = await updateSkill({
        sessionToken,
        id: skill.id,
        enabled: skill.enabled !== 1
      });
      setSkills((prev) => prev.map((item) => (item.id === updated.id ? updated : item)));
    } catch (updateError) {
      setError(updateError instanceof Error ? updateError.message : "Failed to update skill");
    } finally {
      setPendingId(null);
    }
  }

  return (
    <AppShell
      active="skills"
      title="Skills"
      subtitle="View installed skills and control which skills can be used by digital employees."
    >
      <Card>
        <CardHeader>
          <CardTitle>Installed Skills</CardTitle>
          <CardDescription>
            Disabled skills remain installed but cannot be executed in chat and channels.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {loading ? (
            <p className="text-sm text-slate-500">Loading skills...</p>
          ) : skills.length === 0 ? (
            <p className="text-sm text-slate-500">No skills found.</p>
          ) : (
            skills.map((skill) => {
              const isPending = pendingId === skill.id;
              const isEnabled = skill.enabled === 1;
              return (
                <div
                  key={skill.id}
                  className="flex items-center justify-between rounded-lg border border-slate-200 p-3"
                >
                  <div>
                    <p className="text-sm font-semibold text-slate-900">{skill.name}</p>
                    <p className="text-xs text-slate-500">{skill.id}</p>
                  </div>

                  <div className="flex items-center gap-2">
                    <Badge variant={isEnabled ? "default" : "secondary"}>
                      {isEnabled ? "Enabled" : "Disabled"}
                    </Badge>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={isPending}
                      onClick={() => onToggle(skill)}
                    >
                      {isPending ? <Spinner className="h-4 w-4" /> : null}
                      {isEnabled ? "Disable" : "Enable"}
                    </Button>
                  </div>
                </div>
              );
            })
          )}

          {error ? (
            <Alert variant="destructive">
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : null}
        </CardContent>
      </Card>
    </AppShell>
  );
}
