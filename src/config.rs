use anyhow::{Context, Result};
use std::env;

/// .env設定ファイルを格納
#[derive(Debug, Clone)]
pub struct Config {
    // 踏み台サーバ、SSH設定
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub ssh_key_path: String,
    pub local_port: u16,

    // RDS設定
    pub rds_host: String,
    pub rds_port: u16,
    pub database_url: String,

    // ファイルパス
    pub schemas_path: String,
    pub sql_path: String,
    pub output_path: String,

    // クエリ設定
    pub sql_concurrency: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            ssh_host: env::var("SSH_HOST")
                .context("SSH_HOSTが設定されていません")?,
            ssh_port: env::var("SSH_PORT")
                .context("SSH_PORTが設定されていません")?
                .parse()
                .context("SSH_PORTの値が不正です")?,
            ssh_user: env::var("SSH_USER")
                .context("SSH_USERが設定されていません")?,
            ssh_key_path: env::var("SSH_KEY_PATH")
                .context("SSH_KEY_PATHが設定されていません")?,
            local_port: env::var("LOCAL_PORT")
                .context("LOCAL_PORTが設定されていません")?
                .parse()
                .context("LOCAL_PORTの値が不正です")?,

            rds_host: env::var("RDS_HOST")
                .context("RDS_HOSTが設定されていません")?,
            rds_port: env::var("RDS_PORT")
                .context("RDS_PORTが設定されていません")?
                .parse()
                .context("RDS_PORTの値が不正です")?,
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URLが設定されていません")?,

            schemas_path: env::var("SCHEMAS_PATH")
                .context("SCHEMAS_PATHが設定されていません")?,
            sql_path: env::var("SQL_PATH")
                .context("SQL_PATHが設定されていません")?,
            output_path: env::var("OUTPUT_PATH")
                .context("OUTPUT_PATHが設定されていません")?,

            sql_concurrency: env::var("SQL_CONCURRENCY")
                .context("SQL_CONCURRENCYが設定されていません")?
                .parse()
                .context("SQL_CONCURRENCYの値が不正です")?,
        })
    }
}
