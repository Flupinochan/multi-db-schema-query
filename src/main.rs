mod logger;
mod ssh;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use sqlx::mysql::MySqlPool;
use ssh::SshClient;
use tracing::{error, info};

// グローバル定数の定義
const SSH_HOST: &str = "100.48.208.193";
const SSH_PORT: u16 = 22;
const SSH_USER: &str = "ec2-user";
const SSH_KEY_PATH: &str = "multi-db-schema-query.pem";
const LOCAL_PORT: u16 = 3306;
const RDS_HOST: &str = "terraform-20251228151608168400000002.cxfnvhaqo6xk.us-east-1.rds.amazonaws.com";
const RDS_PORT: u16 = 3306;
const DB_URL: &str = "mysql://admin:password1!@localhost:3306/mydb";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // ロガー初期化
    logger::init();

    info!("処理を開始します");
    if let Err(e) = run().await {
        error!("異常終了しました: {e:#}");
        std::process::exit(1);
    }
    info!("正常終了しました");
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

    // ここでMySQLクライアントを使用
    let pool = MySqlPool::connect(DB_URL).await?;

    let now = sqlx::query_scalar::<_, NaiveDateTime>("SELECT NOW()")
        .fetch_one(&pool)
        .await?;

    info!("現在時刻: {}", now);

    pool.close().await;
    // SSHトンネルを終了
    client.stop_tunnel();

    Ok(())
}
