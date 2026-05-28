# ---- 构建阶段 ----
FROM rust:1.87-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/

RUN cargo build --release --all-features

# ---- 运行阶段 ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mutao /usr/local/bin/mutao
COPY migrations/ /app/migrations/

WORKDIR /app
EXPOSE 3000

CMD ["mutao"]
