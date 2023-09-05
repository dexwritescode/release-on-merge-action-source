FROM alpine:3.18

WORKDIR /app

COPY target/debug/roma .

# Run the release on merge action
ENTRYPOINT ["/app/roma"]