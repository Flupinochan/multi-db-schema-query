mod db;
mod logger;
mod ssh;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use sqlx::mysql::MySqlPool;
use ssh::SshClient;
use tracing::{error, info};

// グローバル定数の定義
const SSH_HOST: &str = "13.223.99.141";
const SSH_PORT: u16 = 22;
const SSH_USER: &str = "ec2-user";
const SSH_KEY_PATH: &str = "/home/metalmental/multi-db-schema-query.pem";

const LOCAL_PORT: u16 = 3306;
const RDS_HOST: &str = "terraform-20251229055855571100000001.cxfnvhaqo6xk.us-east-1.rds.amazonaws.com";
const RDS_PORT: u16 = 3306;
const DATABASE_URL: &str = "mysql://admin:password1!@localhost:3306/mydb";

const SCHEMAS_PATH: &str = "resources/schemas.txt";
const SQL_PATH: &str = "resources/query.sql";
const OUTPUT_CSV_PATH: &str = "output/results.csv";
const SQL_CONCURRENCY: usize = 10;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // ロガー初期化
    logger::init();

    info!("処理開始");
    if let Err(e) = run().await {
        error!("異常終了: {e:#}");
        std::process::exit(1);
    }
    info!("正常終了");
}

async fn run() -> Result<()> {
    // SSHクライアント作成
    let mut client =
        SshClient::new(SSH_HOST, SSH_PORT, SSH_USER, SSH_KEY_PATH, None)?;

    // コマンド実行
    let hostname = client.exec_command("hostname")?;
    info!("踏み台サーバ: {}", hostname);

    // 踏み台サーバのEC2にSSHトンネルを開始
    client
        .start_tunnel(LOCAL_PORT, RDS_HOST, RDS_PORT)
        .context("SSHトンネルの開始に失敗しました")?;

    let pool = MySqlPool::connect(DATABASE_URL).await?;

    let now: NaiveDateTime =
        sqlx::query_scalar("SELECT NOW()").fetch_one(&pool).await?;

    info!("現在時刻: {}", now);

    // テスト用のスキーマとテーブルを作成
    db::setup_test_schemas(&pool).await?;

    // スキーマ一覧を取得
    let schemas_path = std::path::Path::new(SCHEMAS_PATH);
    let schemas_content = std::fs::read_to_string(schemas_path)
        .context("スキーマファイルの読み込みに失敗しました")?;
    let schemas: Vec<String> = schemas_content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    // クエリファイルを読み込む
    let sql_path = std::path::Path::new(SQL_PATH);
    let sql = std::fs::read_to_string(sql_path)
        .context("クエリファイルの読み込みに失敗しました")?;

    // 各スキーマに対してクエリを実行
    let schema_refs: Vec<&str> = schemas.iter().map(|s| s.as_str()).collect();
    let results =
        db::query_schemas(&pool, &schema_refs, &sql, SQL_CONCURRENCY).await?;

    // 結果をログ出力とCSV出力
    db::print_rows(&results);
    db::write_csv(&results, std::path::Path::new(OUTPUT_CSV_PATH))?;

    // DB接続を切断
    pool.close().await;
    // SSHトンネルを終了
    client.stop_tunnel();

    Ok(())
}
