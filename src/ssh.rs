use anyhow::{Context, Result};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use tracing::info;

pub struct SshClient {
    host: String,
    port: u16,
    username: String,
    key_path: String,
    timeout: Duration,
    tunnel: Option<SshTunnel>,
}

struct SshTunnel {
    process: Child,
}

impl SshClient {
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        key_path: &str,
        timeout: Option<Duration>,
    ) -> Result<Self> {
        check_ssh_command_available()?;

        Ok(Self {
            host: host.to_string(),
            port,
            username: username.to_string(),
            key_path: key_path.to_string(),
            timeout: timeout.unwrap_or(Duration::from_secs(5)),
            tunnel: None,
        })
    }

    /// リモートサーバーでコマンドを実行し、標準出力を返却
    pub fn exec_command(&self, command: &str) -> Result<String> {
        let output = Command::new("ssh")
            .args([
                "-i",
                &self.key_path,
                "-p",
                &self.port.to_string(),
                "-o",
                "StrictHostKeyChecking=accept-new",
                "-o",
                &format!("ConnectTimeout={}", self.timeout.as_secs()),
                &format!("{}@{}", self.username, self.host),
                command,
            ])
            .output()
            .context("コマンドの実行に失敗しました")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "コマンドの実行に失敗しました (終了コード: {:?}): {}",
                output.status.code().unwrap(),
                stderr.trim()
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// SSHトンネルを開始(バックグラウンドで動作)
    pub fn start_tunnel(
        &mut self,
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<()> {
        // 既存のトンネルがあれば終了
        if self.tunnel.is_some() {
            // Dropを呼び出して終了
            self.tunnel = None;
        }

        // SSHトンネルを起動
        let process = Command::new("ssh")
            .args([
                "-i",
                &self.key_path,
                "-p",
                &self.port.to_string(),
                "-N",
                "-L",
                &format!("{}:{}:{}", local_port, remote_host, remote_port),
                &format!("{}@{}", self.username, self.host),
                "-o",
                "StrictHostKeyChecking=accept-new",
                "-o",
                "ServerAliveInterval=60",
            ])
            .spawn()
            .context("SSHトンネルの起動に失敗しました")?;

        // トンネルが確立されるまで待機
        thread::sleep(Duration::from_secs(2));

        info!(
            "SSHトンネル開始: localhost:{} -> {}:{}",
            local_port, remote_host, remote_port
        );

        self.tunnel = Some(SshTunnel { process });

        Ok(())
    }

    /// SSHトンネルを終了
    pub fn stop_tunnel(&mut self) {
        self.tunnel = None;
    }
}

/// SSHトンネルのクリーンアップ
impl Drop for SshTunnel {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        info!("SSHトンネル終了");
    }
}

/// SSHコマンドが利用可能か確認
fn check_ssh_command_available() -> Result<()> {
    Command::new("ssh").arg("-V").output().context(
        "sshコマンドが見つかりません。OpenSSH をインストールしてください",
    )?;

    Ok(())
}
