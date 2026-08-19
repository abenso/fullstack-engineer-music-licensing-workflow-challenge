# syntax=docker/dockerfile:1

# ---- builder ----
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# sqlx::query!/query_as! validate SQL at compile time, which normally needs
# a live Postgres — not available during `docker build`. SQLX_OFFLINE makes
# them read the checked-in .sqlx/ cache instead (see: cargo sqlx prepare).
ENV SQLX_OFFLINE=true

# Build dependencies first so they're cached separately from application code.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
RUN touch src/main.rs \
    && cargo build --release

# ---- runtime ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/music-licensing-backend ./music-licensing-backend

EXPOSE 8080
CMD ["./music-licensing-backend"]
