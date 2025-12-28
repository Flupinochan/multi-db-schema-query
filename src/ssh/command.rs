use anyhow::{Context, Result};
use ssh2::Session;
use std::io::Read;

/// 対象のセッション(サーバ)でコマンドを実行し、標準出力を返却
///
/// # Errors
///
/// そのまま上位にスロー
pub fn exec_command(session: &Session, command: &str) -> Result<String> {
    let mut channel = session.channel_session()?;
    channel.exec(command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.wait_close()?;

    Ok(output)
}
