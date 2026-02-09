# Build stage
FROM rust:1.82-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p dafhne-server

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/dafhne-server /usr/local/bin/
COPY dictionaries/ /data/dictionaries/
COPY results_multi/ /data/results_multi/

WORKDIR /data
EXPOSE 3000

ENTRYPOINT ["dafhne-server"]
CMD ["--data-dir", "/data/dictionaries", "--port", "3000"]
