FROM alpine:3.18

WORKDIR /app

COPY target/debug/release-on-merge-action /app/
RUN mv release-on-merge-action roma

# Run the release on merge action
ENTRYPOINT ["/app/roma"]