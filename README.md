# Music Licensing Workflow

A backend service for tracking the music licensing process across a movie
catalog: which songs are used in which scenes, and the status of the
rights-holder negotiation for each of those uses.

This repository currently implements the **backend API**. A web client is
not included yet — see [Future work](#future-work).

## Overview

Each movie is made up of scenes, and each scene can use one or more songs
(each usage is called a **track**: a song, a time window within the scene,
and its own licensing status). Licensing a song for a movie involves
back-and-forth negotiation with the rights holder (an artist or a label),
so every track carries a `license_status` that moves through a fixed
workflow, with a full audit trail of every change.

The API also exposes a real-time event stream so that anyone watching a
movie's tracks sees status changes the moment they happen, without polling.

## Architecture at a glance

```mermaid
flowchart LR
    Client -->|REST, JSON| API[Axum API]
    API --> Service[License workflow validation]
    Service --> Repo[SQLx repositories]
    Repo --> DB[(PostgreSQL)]
    Service -.->|on status change| Bus[Broadcast channel]
    Bus -.-> SSE[GET /events]
    SSE -.->|text/event-stream| Client
```

- **[Axum](https://github.com/tokio-rs/axum)** — HTTP framework, built on Tokio.
- **[SQLx](https://github.com/launchbadge/sqlx)** — async Postgres client with compile-time checked queries (no ORM).
- **PostgreSQL** — primary datastore, including a native `license_status` enum.
- **Server-Sent Events** — real-time delivery of license status changes.

See [Tech decisions & trade-offs](#tech-decisions--trade-offs) for the reasoning behind each of these.

## Getting started

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose (bundled with Docker Desktop)

### Run everything with Docker

```bash
docker compose up --build
```

This starts PostgreSQL and the backend together. The backend runs any
pending database migrations automatically on startup — there's no manual
migration step. Once it's up:

```bash
curl http://localhost:8080/health
# ok
```

### Environment variables

The backend reads `DATABASE_URL`, `PORT`, and `RUST_LOG`. When running via
`docker compose`, these are already set in `compose.yml`. For running the
binary directly (outside Docker), copy the example file:

```bash
cp backend/.env.example backend/.env
```

| Variable | Default (in `.env.example`) | Purpose |
|---|---|---|
| `DATABASE_URL` | `postgres://licensing:licensing@localhost:5432/licensing` | Postgres connection string |
| `PORT` | `8080` | HTTP port the server listens on |
| `RUST_LOG` | `info` | Log verbosity ([`tracing-subscriber`](https://docs.rs/tracing-subscriber) filter syntax) |

### Running locally without Docker

Requires a local Rust toolchain and a reachable Postgres (the simplest way
is still `docker compose up -d postgres`, exposed on `localhost:5432`):

```bash
cd backend
cargo run
```

## API reference

All request/response bodies are JSON. Errors follow the shape
`{"error": "<message>"}` with an appropriate HTTP status code (`400`,
`404`, `409`, or `500`).

### Movies

| Method | Path | Description |
|---|---|---|
| `POST` | `/movies` | Create a movie |
| `GET` | `/movies` | List all movies |
| `GET` | `/movies/{id}` | Get a movie by id |

```bash
curl -X POST http://localhost:8080/movies \
  -H "Content-Type: application/json" \
  -d '{"title": "Rocky IV"}'
```

### Scenes

| Method | Path | Description |
|---|---|---|
| `POST` | `/movies/{id}/scenes` | Create a scene under a movie |
| `GET` | `/movies/{id}/scenes` | List a movie's scenes |

```bash
curl -X POST http://localhost:8080/movies/$MOVIE_ID/scenes \
  -H "Content-Type: application/json" \
  -d '{"name": "Training Montage"}'
```

### Songs

| Method | Path | Description |
|---|---|---|
| `POST` | `/songs` | Add a song to the catalog |
| `GET` | `/songs` | List all songs |

```bash
curl -X POST http://localhost:8080/songs \
  -H "Content-Type: application/json" \
  -d '{"title": "Eye of the Tiger", "artist": "Survivor", "rights_holder": "Sony"}'
```

A song is catalog data (title, artist, rights holder) and can be reused
across multiple tracks — see [Data model](#data-model) for why songs and
tracks are separate.

### Tracks

| Method | Path | Description |
|---|---|---|
| `POST` | `/scenes/{id}/tracks` | Add a track (a song's usage) to a scene |
| `GET` | `/scenes/{id}/tracks` | List a scene's tracks, with each track's song |
| `GET` | `/movies/{id}/tracks` | List every track in a movie, grouped by scene |
| `GET` | `/tracks/{id}` | Get a track's detail, including its full license history |
| `PATCH` | `/tracks/{id}/license` | Move a track's license status |

```bash
curl -X POST http://localhost:8080/scenes/$SCENE_ID/tracks \
  -H "Content-Type: application/json" \
  -d '{"song_id": "'$SONG_ID'", "start_time_ms": 750000, "end_time_ms": 840000}'
```

`start_time_ms`/`end_time_ms` mark the song's window within the scene;
`end_time_ms` must be greater than `start_time_ms` (`400` otherwise). A new
track starts in the `Draft` license status.

### Licensing workflow

See [`docs/license-workflow.md`](docs/license-workflow.md) for the full
transition table and what each move means.

```mermaid
stateDiagram-v2
    [*] --> Draft
    Draft --> Requested
    Requested --> InNegotiation
    Requested --> Rejected
    InNegotiation --> Approved
    InNegotiation --> Rejected
    Approved --> Licensed
    Licensed --> [*]
    Rejected --> [*]
```

`Licensed` and `Rejected` are terminal. Any other transition (skipping a
step, or moving backwards) is rejected:

```bash
curl -X PATCH http://localhost:8080/tracks/$TRACK_ID/license \
  -H "Content-Type: application/json" \
  -d '{"status": "Requested", "note": "Sent to Sony legal team"}'
```

A successful move returns the updated track, including the appended
history entry:

```json
{
  "id": "…",
  "song": { "id": "…", "title": "Eye of the Tiger", "artist": "Survivor" },
  "start_time_ms": 750000,
  "end_time_ms": 840000,
  "license_status": "Requested",
  "updated_at": "2026-08-19T18:05:14Z",
  "history": [
    {
      "from_status": "Draft",
      "to_status": "Requested",
      "note": "Sent to Sony legal team",
      "created_at": "2026-08-19T18:05:14Z"
    }
  ]
}
```

An illegal move (e.g. `Draft` → `Licensed`) returns `409 Conflict` with a
message naming the offending states, instead of a generic error.

### Real-time updates

```
GET /events
```

Opens a persistent `text/event-stream` connection. Every time a track's
license status changes, all connected clients receive:

```
event: license_status_changed
data: {"track_id":"…","scene_id":"…","from":"Draft","to":"Requested","at":"2026-08-19T18:05:14Z"}
```

```bash
curl -N http://localhost:8080/events
```

## Data model

```mermaid
erDiagram
    MOVIE ||--o{ SCENE : has
    SCENE ||--o{ TRACK : has
    SONG ||--o{ TRACK : "used in"
    TRACK ||--o{ LICENSE_STATUS_EVENT : audits
```

A `Track` is a song's use within one scene — it holds the time window and
the license status *of that specific use*, not of the song in general.
`Song` is separate, reusable catalog data: the same song can appear in
several tracks (different scenes, or a movie reusing it in the credits),
each with its own independent negotiation. `license_status_events` is an
append-only audit log: one row per transition, so the full negotiation
history for a track is always available, not just its current state.

## Tech decisions & trade-offs

| Decision | Choice | Why |
|---|---|---|
| Web framework | Axum | Built on Tokio, the de facto standard async runtime; simpler surface than Actix-web while still integrating cleanly with `tower` middleware. |
| Database access | SQLx, offline mode | SQL is written explicitly (no ORM query builder to reason about), but `query!`/`query_as!` still validate every query against the real schema at compile time. Offline mode (query metadata cached in `backend/.sqlx/`, checked into the repo) means `docker build` and CI don't need a live database just to compile. |
| API style | REST | The domain is CRUD plus state transitions on a handful of resources — GraphQL's flexible querying doesn't add value here, and REST keeps the surface simple to review. |
| Real-time | Server-Sent Events | One-way server→client push is all this needs. SSE runs over plain HTTP (no extra protocol/handshake like WebSockets, no subscription infrastructure like GraphQL subscriptions) and Axum supports it directly. |
| License status storage | Postgres native `enum` + a `license_status_events` audit table | The enum rejects invalid values at the database level, not just in application code. The audit table (rather than only a "current status" column) preserves the full negotiation history practically for free. |
| Error handling | A single `AppError` enum implementing `IntoResponse` | One place mapping domain/DB failures to HTTP status codes, instead of repeating `match` logic in every handler. |
| Testing | Unit tests for pure logic (the license state machine, time-range validation, error mapping) + HTTP-level integration tests (`tower::ServiceExt::oneshot` against the real `Router`, with `#[sqlx::test]` giving each test an isolated, auto-migrated database) | Fast, DB-free tests for business rules; realistic tests for everything that touches the database, without hand-rolled mocks. |
| Concurrency safety | Compare-and-swap on the license status update (`UPDATE ... WHERE license_status = <expected>`), in the same transaction as the audit event insert | Two concurrent `PATCH` requests on the same track must not both apply against the same stale read — one has to lose cleanly (`409`) instead of silently overwriting the other or leaving the audit trail out of sync. `tests/license_concurrency_test.rs` simulates this deterministically (two callers with the same stale read, calling the repository directly) rather than spawning real concurrent HTTP requests, which would be non-deterministic in CI. The fix was additionally exercised once, by hand, with a real 20-concurrent-request `curl` load test against a running instance — not part of the automated suite, but confirmed the same behavior under real concurrency. |
| CORS | `tower_http::cors::CorsLayer::permissive()` | The API contract is meant to be called from a browser-based frontend on a different origin; without CORS headers, the browser blocks the request before it reaches the handlers. No auth/cookies exist yet to warrant restricting it further. |

## Testing

```bash
docker compose up -d postgres
cd backend
DATABASE_URL="postgres://licensing:licensing@localhost:5432/licensing" cargo test
```

33 tests: 17 unit tests (no database needed) and 16 integration tests
(each running against its own isolated, freshly migrated database).

Measured with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)
(unit + integration together): **~89% line coverage**. The only
uncovered files are `config.rs`, `db.rs`, and `main.rs` — wiring and the
process entrypoint, with no branching logic to test, exercised in practice
by `docker compose up`.

```bash
cargo install cargo-llvm-cov
DATABASE_URL="postgres://licensing:licensing@localhost:5432/licensing" cargo llvm-cov --summary-only
```

## CI/CD

`.github/workflows/backend-ci.yml` runs on every pull request and push to
`main`/`dev`:

1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo llvm-cov` — runs the full test suite and **fails the build if
   line coverage drops below 80%**; posts a short summary as a PR comment
   (updated in place on every push) and the full per-file breakdown in the
   run's Job Summary.

## Assumptions & constraints

- **License status changes are manual.** There is no integration with an
  external rights-management system — the negotiation itself happens
  outside this service (email, phone, meetings with the rights holder).
  The API is the internal tracker for that process: a person updates a
  track's status as the real negotiation progresses, and the service
  enforces that the update is a legal step in the workflow.
- **No authentication.** Every endpoint is open. Adding auth was left out
  to keep the scope focused — see [Future work](#future-work).
- **Movie titles are not unique.** `POST /movies` always creates a new
  record, even with a duplicate title, the same as any REST `create`
  endpoint. Catching accidental duplicates is treated as a UX concern
  (the frontend warning before submitting), not a database constraint —
  a `UNIQUE` constraint would also incorrectly block legitimate
  same-title entries (e.g. a remake).

## Future work

- **Frontend.** A React client consuming this API — the contract above
  (in particular `GET /movies/{id}/tracks` and the `/events` stream) was
  designed with that client in mind, even though it isn't built here.
- **Authentication/authorization.** Currently out of scope; would sit as
  middleware in front of the existing handlers without changing them.
- **GraphQL.** Not needed today (see [Tech decisions](#tech-decisions--trade-offs)), but could be added alongside REST if a frontend needed more flexible querying.
- **Kafka/Redis.** The event bus is an in-process `tokio::sync::broadcast`
  channel, which only fans out within a single backend instance. Running
  multiple instances behind a load balancer would need a shared bus
  (Redis pub/sub, or Kafka) so an event published on one instance reaches
  clients connected to another.
- **Guaranteed SSE delivery.** A client that lags far enough behind
  currently just misses the events it couldn't keep up with (the
  broadcast channel is bounded), rather than replaying them. Acceptable
  today since a client can always re-fetch full state with a `GET`; would
  need a persisted, offset-based event log to guarantee delivery.
- **Rights-management integration.** If a real external system existed,
  it could call into the license workflow via a webhook instead of a
  person doing it manually — see [Assumptions](#assumptions--constraints).
- **Pagination.** The `GET` list endpoints (`/movies`, `/songs`, and the
  track listings) return everything in one response. Fine at today's
  scale; a catalog large enough for this to matter would need cursor- or
  offset-based pagination.

## Project structure

```
backend/
├── migrations/            # SQL migrations (run automatically on startup)
├── src/
│   ├── domain/             # Entities + the license state machine (pure, unit-tested)
│   ├── repositories/       # SQLx queries, one module per table
│   ├── handlers/           # Axum request handlers
│   ├── dto.rs               # Response shapes that join multiple tables
│   ├── error.rs             # AppError -> HTTP response mapping
│   ├── events.rs            # Broadcast channel for real-time updates
│   ├── routes.rs            # Route table
│   ├── db.rs                 # Connection pool + migration runner
│   ├── config.rs             # Environment-based configuration
│   ├── lib.rs                 # Library root (used by main.rs and by tests/)
│   └── main.rs                 # Entrypoint
├── tests/                  # HTTP-level integration tests
└── Dockerfile
compose.yml                 # App + Postgres, for local development
```
