# ビルドステージ
FROM rust AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# ランタイムステージ
FROM debian:trixie-slim

WORKDIR /app

# SSHコマンドをインストール
RUN apt-get update && \
    apt-get install -y openssh-client && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/multi-db-schema-query ./
COPY multi-db-schema-query.pem ./
COPY ./sql/query.sql ./sql/query.sql

RUN chmod 400 multi-db-schema-query.pem

CMD ["./multi-db-schema-query"]