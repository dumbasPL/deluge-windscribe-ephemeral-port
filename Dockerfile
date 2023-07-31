FROM rust:1-bookworm as chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin windscribe-ephemeral-port

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /config
RUN groupadd --gid 1000 wind && useradd --uid 1000 --gid wind --shell /bin/bash wind && chown wind:wind /config
COPY --from=builder /app/target/release/windscribe-ephemeral-port /usr/local/bin
USER wind
ENTRYPOINT ["/usr/local/bin/windscribe-ephemeral-port", "--cache-dir", "/config", "--config", "/config/config.yaml"]
