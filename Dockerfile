FROM alpine:3.18

WORKDIR /app

COPY target/release/roma .

# Run the release on merge action
ENTRYPOINT ["/app/roma"]