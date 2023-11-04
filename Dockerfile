FROM rust:1.73-bookworm as builder

WORKDIR /release-on-merge-action
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime
COPY --from=builder /release-on-merge-action/target/release/release-on-merge-action /app/release-on-merge-action
ENTRYPOINT ["/app/release-on-merge-action"]