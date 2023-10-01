FROM rust:1.72 as builder
WORKDIR /usr/src/release-on-merge-action
COPY . .
RUN cargo install --path .

FROM rust:1.72
COPY --from=builder /usr/local/cargo/bin/release-on-merge-action /app/release-on-merge-action
ENTRYPOINT ["/app/release-on-merge-action"]