# ビルドステージ
FROM rust:latest AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssh2-1-dev \
    && rm -rf /var/lib/apt/lists/*

ENV LIBSSH2_SYS_USE_PKG_CONFIG=1

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

# ランタイムステージ
FROM rust:slim

WORKDIR /app

# 実行に必要なライブラリ
RUN apt-get update && apt-get install -y \
    libssh2-1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/multi-db-schema-query ./
COPY multi-db-schema-query.pem ./

RUN chmod 400 multi-db-schema-query.pem

CMD ["./multi-db-schema-query"]