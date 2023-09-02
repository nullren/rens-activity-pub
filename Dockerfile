FROM rust:1-bookworm AS base
WORKDIR /app

FROM base AS builder
COPY . .
RUN cargo build --release -p rap-server

FROM debian:bookworm-slim AS runtime
RUN apt update -y && apt install -y ca-certificates
WORKDIR app
COPY --from=builder /app/target/release/rap-server /usr/local/bin
CMD ["/usr/local/bin/rap-server"]
