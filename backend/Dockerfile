# syntax=docker/dockerfile:1

# ---- builder ----
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# Build dependencies first so they're cached separately from application code.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN touch src/main.rs \
    && cargo build --release

# ---- runtime ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/music-licensing-backend ./music-licensing-backend

EXPOSE 8080
CMD ["./music-licensing-backend"]
