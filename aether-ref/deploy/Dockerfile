FROM rust:1.75 AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        iproute2 ethtool iputils-ping curl ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/aether /usr/local/bin/aether
EXPOSE 8080
ENTRYPOINT ["aether"]
CMD ["serve", "--bind", "0.0.0.0:8080"]
