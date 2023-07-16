FROM rust:1-bookworm as builder

WORKDIR /app

COPY . .

RUN cargo build --release --bin windscribe-ephemeral-port

FROM debian:bookworm-slim AS runtime

RUN groupadd --gid 1000 wind && useradd --uid 1000 --gid wind --shell /bin/bash --create-home wind

WORKDIR /app

COPY --from=builder /app/target/release/windscribe-ephemeral-port /usr/local/bin

USER wind

ENTRYPOINT ["/usr/local/bin/windscribe-ephemeral-port"]
