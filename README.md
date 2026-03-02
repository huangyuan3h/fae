# fae MVP

Local-first AI assistant daemon with a separated Next.js frontend.

## Prerequisites

- Bun `1.0+`
- Local Ollama runtime (optional for full chat responses)

## Development

```bash
# Backend daemon
bun run dev:daemon

# Frontend web app
bun run dev:web
```

Backend API runs at `http://127.0.0.1:8787` by default and frontend runs at `http://localhost:3000`.

If you need a custom backend port, set `DAEMON_PORT` (daemon) and `NEXT_PUBLIC_DAEMON_PORT` (web) to the same value:

```bash
DAEMON_PORT=18080 NEXT_PUBLIC_DAEMON_PORT=18080 bun dev
```

## Build

```bash
# Build backend binary
bun run build:daemon

# Build frontend
bun run build:web
```
