# syntax=docker/dockerfile:1

# chefイメージを利用
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
ENV RUSTC_WRAPPER=sccache \
    SCCACHE_DIR=/sccache \
    RUSTFLAGS="-C link-arg=-fuse-ld=mold"
ARG SCCACHE_VERSION="0.12.0"
ARG MOLD_VERSION="2.40.4"

WORKDIR /app

# sccacheのインストール
RUN ARCH=$(uname -m) \
    && curl -fL -O https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-${ARCH}-unknown-linux-musl.tar.gz \
    && tar xzf sccache-* \
    && cp -p sccache-*/sccache /usr/local/bin/ \
    && rm -rf sccache-*

# moldのインストール
RUN ARCH=$(uname -m) \
    && curl -fL -O https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-${ARCH}-linux.tar.gz \
    && tar xzf mold-* \
    && cp -p mold-*/bin/* /usr/local/bin/ \
    && rm -rf mold-*

# cargo chefで依存関係を構築
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ビルドステージ
FROM chef AS builder
# 依存関係のクレートをビルド&キャッシュ
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=${SCCACHE_DIR},sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# アプリ本体をビルド&キャッシュ
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=${SCCACHE_DIR},sharing=locked \
    cargo build --release --bin multi-db-schema-query

# ランタイムステージ
FROM debian:trixie-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/multi-db-schema-query /usr/local/bin

# SSHトンネル用にSSHコマンドをインストール
RUN apt-get update && \
    apt-get install -y openssh-client && \
    rm -rf /var/lib/apt/lists/*

COPY ./resources/query.sql ./resources/query.sql
COPY ./resources/schemas.txt ./resources/schemas.txt
COPY ./multi-db-schema-query.pem ./multi-db-schema-query.pem

RUN chmod 400 multi-db-schema-query.pem

# 一般ユーザ設定
RUN groupadd -g 1001 appgroup && \
    useradd -u 1001 -g appgroup -m -d /home/appuser -s /bin/bash appuser
USER appuser

ENTRYPOINT ["/usr/local/bin/multi-db-schema-query"]