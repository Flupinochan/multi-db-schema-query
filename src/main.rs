mod config;
mod db;
mod logger;
mod ssh;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use config::Config;
use sqlx::mysql::MySqlPool;
use ssh::SshClient;
use tracing::{error, info};

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
    // 設定ファイル読み込み
    let config =
        Config::from_env().context("設定ファイルの読み込みに失敗しました")?;

    // SSHクライアント作成
    let mut client = SshClient::new(
        &config.ssh_host,
        config.ssh_port,
        &config.ssh_user,
        &config.ssh_key_path,
        None,
    )?;
    // コマンド実行
    let hostname = client.exec_command("hostname")?;
    info!("踏み台サーバ: {}", hostname);

    // 踏み台サーバのEC2にSSHトンネルを開始
    client
        .start_tunnel(config.local_port, &config.rds_host, config.rds_port)
        .context("SSHトンネルの開始に失敗しました")?;

    let pool = MySqlPool::connect(&config.database_url).await?;

    let now: NaiveDateTime =
        sqlx::query_scalar("SELECT NOW()").fetch_one(&pool).await?;

    info!("現在時刻: {}", now);

    // テスト用のスキーマとテーブルを作成
    db::setup_test_schemas(&pool).await?;

    // スキーマ一覧を取得
    let schemas_path = std::path::Path::new(&config.schemas_path);
    let schemas_content = std::fs::read_to_string(schemas_path)
        .context("スキーマファイルの読み込みに失敗しました")?;
    let schemas: Vec<String> = schemas_content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    // クエリファイルを読み込む
    let sql_path = std::path::Path::new(&config.sql_path);
    let sql = std::fs::read_to_string(sql_path)
        .context("クエリファイルの読み込みに失敗しました")?;

    // 各スキーマに対してクエリを実行
    let schema_refs: Vec<&str> = schemas.iter().map(|s| s.as_str()).collect();
    let results =
        db::query_schemas(&pool, &schema_refs, &sql, config.sql_concurrency)
            .await?;

    // 結果をログ出力とCSV出力
    db::print_rows(&results);
    db::write_csv(&results, std::path::Path::new(&config.output_path))?;

    // DB接続を切断
    pool.close().await;
    // SSHトンネルを終了
    client.stop_tunnel();

    Ok(())
}
