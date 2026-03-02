export const SESSION_KEY = "fae_session_token";

export function getSessionToken(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  return window.localStorage.getItem(SESSION_KEY);
}

export function saveSessionToken(token: string): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.setItem(SESSION_KEY, token);
}

export function clearSessionToken(): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.removeItem(SESSION_KEY);
}

export async function ensureSessionToken(
  createToken: () => Promise<string>
): Promise<string> {
  const existing = getSessionToken();
  if (existing) {
    return existing;
  }

  const created = await createToken();
  saveSessionToken(created);
  return created;
}
