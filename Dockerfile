FROM alpine:3.10

WORKDIR /app

COPY target/debug/release-on-merge-action /app/roma

# Build the releaseaction here
ENTRYPOINT ["/app/roma"]