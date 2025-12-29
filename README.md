## ビルド、実行手順

### 以下の `.env` ファイルを作成

#### terraformの場合

```ini
# 踏み台サーバ、SSH設定
SSH_HOST=13.223.99.141
SSH_PORT=22
SSH_USER=ec2-user
SSH_KEY_PATH=multi-db-schema-query.pem
LOCAL_PORT=3306

# RDS設定
RDS_HOST=terraform-20251229055855571100000001.cxfnvhaqo6xk.us-east-1.rds.amazonaws.com
RDS_PORT=3306
DATABASE_URL=mysql://admin:password1!@localhost:3306/mydb

# ファイルパス
SCHEMAS_PATH=resources/schemas.txt
SQL_PATH=resources/query.sql
OUTPUT_PATH=./output/results.csv

# クエリ設定
SQL_CONCURRENCY=10
```

#### docker composeの場合

```ini
# 踏み台サーバ、SSH設定
SSH_HOST=localhost
SSH_PORT=2222
SSH_USER=ec2-user
SSH_KEY_PATH=multi-db-schema-query.pem
LOCAL_PORT=13306

# RDS設定
RDS_HOST=mysql
RDS_PORT=3306
DATABASE_URL=mysql://root:password1!@localhost:13306/mydb

# ファイルパス
SCHEMAS_PATH=resources/schemas.txt
SQL_PATH=resources/query.sql
OUTPUT_PATH=./output/results.csv

# クエリ設定
SQL_CONCURRENCY=10

# 公開鍵の内容を記載 openssh-serverコンテナにコピーされる
PUBLIC_KEY=ssh-rsa ABCDE...
```

### ビルド&実行

```bash
cargo build --release
./target/release/multi-db-schema-query
```
