mod config;
mod db;
mod logger;
mod ssh;

use anyhow::{Context, Result};
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
        error!("異常終了: {e:?}");
        std::process::exit(1);
    }
    info!("正常終了");
}

async fn run() -> Result<()> {
    // .envファイル読み込み
    let config =
        Config::from_env().context(".envファイルの読み込みに失敗しました")?;

    // SSHクライアント初期化
    let mut client = SshClient::new(
        &config.ssh_host,
        config.ssh_port,
        &config.ssh_user,
        &config.ssh_key_path,
        None,
    )?;

    // hostnameコマンド実行 (踏み台サーバへの接続確認)
    let hostname = client
        .exec_command("hostname")
        .context("SSHコマンドの実行に失敗しました")?;
    info!("踏み台サーバ: {}", hostname);

    client
        .start_tunnel(config.local_port, &config.rds_host, config.rds_port)
        .context("SSHトンネルの開始に失敗しました")?;

    let pool = MySqlPool::connect(&config.database_url)
        .await
        .context("DBへの接続に失敗しました")?;

    db::get_current_timestamp(&pool)
        .await
        .context("DBから現在時刻の取得に失敗しました")?;

    // テスト用のスキーマとテーブルを作成
    db::setup_test_schemas(&pool)
        .await
        .context("テスト用スキーマおよびテーブルの作成に失敗しました")?;

    // スキーマ一覧を読み込む
    let schemas_path = std::path::Path::new(&config.schemas_path);
    let schemas_content =
        std::fs::read_to_string(schemas_path).with_context(|| {
            format!(
                "スキーマファイルの読み込みに失敗しました: {}",
                schemas_path.display()
            )
        })?;
    let schemas: Vec<String> = schemas_content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
    let schemas_refs: Vec<&str> = schemas.iter().map(|s| s.as_str()).collect();

    // クエリファイルを読み込む
    let sql_path = std::path::Path::new(&config.sql_path);
    let sql = std::fs::read_to_string(sql_path).with_context(|| {
        format!(
            "クエリファイルの読み込みに失敗しました: {}",
            sql_path.display()
        )
    })?;

    // 各スキーマに対してクエリを実行
    let results =
        db::query_schemas(&pool, &schemas_refs, &sql, config.sql_concurrency)
            .await?;

    // 結果をログ出力とCSV出力
    db::print_rows(&results);
    db::write_csv(&results, std::path::Path::new(&config.output_path))?;

    // DB接続終了
    pool.close().await;

    // SSHトンネル終了
    client.stop_tunnel();

    Ok(())
}
