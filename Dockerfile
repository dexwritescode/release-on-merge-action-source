FROM alpine:3.10

WORKDIR /app

COPY target/debug/release-on-merge-action .

# Build the releaseaction here
ENTRYPOINT ["release-on-merge-action"]