FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p rap-server

FROM debian:bookworm-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/rap-server /usr/local/bin

ENV RUST_LOG=debug
CMD ["/usr/local/bin/rap-server"]
