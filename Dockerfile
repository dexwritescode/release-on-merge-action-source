FROM rust:1.72-bookworm as builder
WORKDIR /usr/src/release-on-merge-action
COPY . .

# Only build in release mode on the release branch
#RUN cargo install --path .
RUN cargo install --debug --path .


FROM gcr.io/distroless/cc-debian12 AS runtime
COPY --from=builder /usr/local/cargo/bin/release-on-merge-action /app/release-on-merge-action
ENTRYPOINT ["/app/release-on-merge-action"]