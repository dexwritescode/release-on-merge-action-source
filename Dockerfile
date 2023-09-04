FROM alpine:3.10

COPY . /builddir/

# Build the releaseaction here
ENTRYPOINT ["/releaseaction"]