FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY rap-server .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY rap-server .
RUN cargo build --release -p rap-server

FROM debian:bullseye-slim AS runtime
RUN apt update -y && apt install -y ca-certificates
WORKDIR app
COPY --from=builder /app/target/release/rap-server /usr/local/bin
CMD ["/usr/local/bin/rap-server"]
