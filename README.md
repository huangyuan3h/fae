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

Backend API runs at `http://127.0.0.1:8080` and frontend runs at `http://localhost:3000`.

## Build

```bash
# Build backend binary
bun run build:daemon

# Build frontend
bun run build:web
```
