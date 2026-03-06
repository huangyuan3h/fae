# FAE Agent

Rust-based backend service for the FAE platform, designed to replace the original TypeScript daemon.

## Endpoints (Replaces @fae/daemon functionality)
- `GET /health` - Health check
- `GET /api/status` - Service status
- `POST /api/chat/stream` - Chat stream endpoint
- `GET /api/settings/providers` - Get provider configurations
- `PUT /api/settings/providers` - Update provider configurations  
- `GET /api/settings/ollama` - Get Ollama settings
- `PUT /api/settings/ollama` - Update Ollama settings

## Features
- High-performance async runtime with Tokio
- Modern web framework with Axum
- Type-safe database access with SQLx and SQLite
- Support for 4 AI providers: Ollama, OpenAI, Google, Alibaba
- Full settings management API compatible with original daemon
- WebSocket support prepared for AI streaming
- Structured logging

## Running locally
```bash
cargo run # Runs on port 3001 by default
```

Or with environment variables:
```bash
DATABASE_URL=sqlite:local.db HOST=0.0.0.0 PORT=3001 cargo run
```

## Development with Frontend
Use the top-level `npm run dev` from the FAE project root to run both Rust agent and Next.js frontend simultaneously:
```bash
# From project root
npm run dev
```