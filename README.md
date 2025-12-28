## Rust バージョン管理

```bash
# https://releases.rs/
rustup install stable
rustup install nightly

# 特定バージョン
rustup install 1.75.0

# グローバルのデフォルト変更
rustup default stable

# 現在のディレクトリのみ指定(rust-toolchain.tomlで管理)
rustup override set stable

# インストール済みを確認
rustup show

# インストール可能なバージョンをlistすることはできない

# Rust コンパイラのバージョン
rustc --version
```

## Cargo ビルド/実行

```bash
# プロジェクト初期化
cargo init .

# ビルド + 実行
cargo run -q
cargo run --release

# テスト実行
cargo test

# ドキュメント生成
cargo doc --open

# Cargo.toml に依存関係を追加
cargo add serde

# 依存関係の更新
cargo update

# 不要なビルド成果物を削除
cargo clean

# リンター
cargo clippy
# フォーマッター
cargo fmt
# 構文チェック
cargo check
```

[Library List](https://lib.rs/)
[Rustfmt](https://rust-lang.github.io/rustfmt/?version=v1.8.0&search=)

## localstack aws

- [desktop](https://apps.microsoft.com/detail/9ntrnft9zws2?hl=ja-JP&gl=JP)
- [docker extention](https://docs.localstack.cloud/aws/tooling/localstack-docker-extension/)
- [vscode extention](https://marketplace.visualstudio.com/items?itemName=localstack.localstack)

```bash
localstack start -d

localstack stop

localstack auth set-token xxxxx
```

### tflocal (terraform)

LocalStack用のTerraformラッパー

```bash
# format
tflocal fmt
# download provider
tflocal init
# validate
tflocal validate

# 変更内容を確認
tflocal plan
# deploy
tflocal apply
tflocal apply -auto-approve

# show status
tflocal show
tflocal state list
tflocal output

# 同期
tflocal refresh

# destroy
tflocal plan -destroy
tflocal destory
tflocal destory -auto-approve

# 依存関係の可視化
tflocal graph
```

### awslocal

[awslocal](https://github.com/localstack/awscli-local)

### 接続

```bash
# EC2
ssh -i .\multi-db-schema-query.pem ec2-user@13.220.45.45

# RDS
mysql -h terraform-20251226203317686000000002.cxfnvhaqo6xk.us-east-1.rds.amazonaws.com -u admin -ppassword1!
```

## Library

sqlxはcliもインストール

```bash
# sql
cargo install sqlx-cli
```

## Docker

```bash
docker build -t multi-db-schema-query-app .
docker run --rm multi-db-schema-query-app
```

## GPG Key

```bash
& "C:\Program Files\Git\usr\bin\gpg.exe" --full-generate-key
```