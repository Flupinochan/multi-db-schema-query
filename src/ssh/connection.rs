use anyhow::{Context, Result};
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;

/// SSH接続のセッションを作成
///
/// # Errors
///
/// - ホストへの通信に失敗した場合
/// - 鍵ファイルによる認証に失敗した場合
pub fn create_session(
    ipaddress: &str,
    port: u16,
    username: &str,
    key_path: &str,
) -> Result<Session> {
    let host = format!("{}:{}", ipaddress, port);
    let tcp = TcpStream::connect(&host)
        .with_context(|| format!("{} への通信に失敗しました", host))?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;
    session
        .userauth_pubkey_file(username, None, Path::new(key_path), None)
        .with_context(|| {
            format!("鍵ファイル {} による認証が失敗しました", key_path)
        })?;
    Ok(session)
}
