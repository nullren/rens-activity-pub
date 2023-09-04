FROM rust:1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p rap-server

FROM debian:bookworm-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/rap-server /usr/local/bin

ENV RUST_LOG=debug
ENV DOMAIN=ap.rens.page
CMD ["/usr/local/bin/rap-server"]
