mod ssh;

use anyhow::{Context, Result};
use ssh::create_session;
use ssh::exec_command;

fn main() -> Result<()> {
    let session = create_session(
        "98.86.166.235",
        22,
        "ec2-user",
        "multi-db-schema-query.pem",
    )
    .context("SSH接続の確立に失敗しました")?;

    println!(
        "{} で処理をします",
        exec_command(&session, "hostname")
            .context("コマンド実行に失敗しました")?
    );

    Ok(())
}
