FROM rust:1.72-bookworm as builder

RUN USER=root cargo new --bin release-on-merge-action
WORKDIR /release-on-merge-action
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
#RUN cargo build --release
RUN cargo build
RUN rm src/*.rs

COPY ./src ./src

# Only build in release mode on the release branch
#RUN rm ./target/release/deps/release_on_merge_action*
#RUN cargo build --release
RUN rm ./target/debug/deps/release_on_merge_action*
RUN cargo build


FROM gcr.io/distroless/cc-debian12 AS runtime
#COPY --from=builder /release-on-merge-action/target/release/release-on-merge-action /app/release-on-merge-action
COPY --from=builder /release-on-merge-action/target/debug/release-on-merge-action /app/release-on-merge-action
ENTRYPOINT ["/app/release-on-merge-action"]