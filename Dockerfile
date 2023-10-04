FROM rust:1.72-slim-bookworm as builder
WORKDIR /usr/src/release-on-merge-action
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim AS runtime
COPY --from=builder /usr/local/cargo/bin/release-on-merge-action /app/release-on-merge-action
ENTRYPOINT ["/app/release-on-merge-action"]