use anyhow::{Context, Result};
use ssh2::Session;
use std::io::Read;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::thread;

pub struct SshClient {
    session: Arc<Session>,
}

impl SshClient {
    /// SSH接続のセッションを作成
    ///
    /// # Errors
    ///
    /// - ホストへの通信に失敗した場合
    /// - 鍵ファイルによる認証に失敗した場合
    pub fn new(
        ipaddress: &str,
        port: u16,
        username: &str,
        key_path: &str,
    ) -> Result<Self> {
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
        Ok(Self {
            session: Arc::new(session),
        })
    }

    /// 対象のセッション(サーバ)でコマンドを実行し、標準出力を返却
    ///
    /// # Errors
    ///
    /// そのまま上位にスロー
    pub fn exec_command(&self, command: &str) -> Result<String> {
        let mut channel: ssh2::Channel = self.session.channel_session()?;
        channel.exec(command)?;

        let mut output: String = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;

        Ok(output)
    }

    /// SSHトンネルを開始(バックグラウンドで動作)
    ///
    /// # Errors
    ///
    ///
    pub fn start_tunnel(
        &self,
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<()> {
        let session = Arc::clone(&self.session);
        let remote_host = remote_host.to_string();

        thread::spawn(move || {
            let listener =
                TcpListener::bind(format!("127.0.0.1:{}", local_port))
                    .expect("ローカルポートのバインドに失敗");

            println!(
                "SSHトンネル開始: localhost:{} -> {}:{}",
                local_port, remote_host, remote_port
            );

            for stream in listener.incoming() {
                if let Ok(mut local_stream) = stream {
                    let session = Arc::clone(&session);
                    let remote_host = remote_host.clone();

                    thread::spawn(move || {
                        if let Ok(mut channel) = session.channel_direct_tcpip(
                            &remote_host,
                            remote_port,
                            None,
                        ) {
                            let mut local_clone =
                                local_stream.try_clone().unwrap();
                            let mut channel_clone =
                                channel.try_clone().unwrap();

                            thread::spawn(move || {
                                std::io::copy(&mut local_stream, &mut channel)
                                    .ok();
                            });
                            thread::spawn(move || {
                                std::io::copy(
                                    &mut channel_clone,
                                    &mut local_clone,
                                )
                                .ok();
                            });
                        }
                    });
                }
            }
        });

        // トンネルが確立されるまで少し待つ
        thread::sleep(std::time::Duration::from_millis(100));

        Ok(())
    }
}
