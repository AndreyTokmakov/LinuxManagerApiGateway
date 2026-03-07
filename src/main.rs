#![allow(
    dead_code,
    unused_imports,
    unused_parens,
    unused_variables,
    non_snake_case
)]

mod ssh_pool_impl
{
    use std::env;
    use std::fs::File;
    use async_trait::async_trait;
    use deadpool::managed::{
        Manager, Metrics, Object, Pool,
        RecycleError, RecycleResult, PoolError
    };
    use ssh2::{Channel, Session};
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, MutexGuard};
    use std::thread::JoinHandle;
    use std::time::Duration;
    use thiserror::Error;

    type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Result<T> = std::result::Result<T, DynError>;

    pub struct SshConnection {
        session: Session,
    }

    pub struct CommandResult
    {
        pub stdout: String,
        pub stderr: String,
        pub exitCode: i32,
    }

    pub struct SshManager
    {
        host: String,
        port: u16,
        user: String,
        password: String,
        private_key: PathBuf,
    }

    impl SshManager
    {
        pub fn new(host: &str,
                   port: u16,
                   user: &str,
                   password: &str,
                   private_key_path: PathBuf) -> Self
        {
            Self {
                host: host.to_string(),
                port,
                user: user.to_string(),
                password: password.to_string(),
                private_key: private_key_path
            }
        }
    }

    impl SshConnection
    {
        pub fn establishSshConnection(host: &str,
                                      port: u16,
                                      user: &str,
                                      password: &str,
                                      private_key_path: &Path) -> Result<Self>
        {
            let tcp: TcpStream = TcpStream::connect((host, port))?;
            tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(10)))?;

            let mut sshSession: Session = Session::new()?;
            sshSession.set_tcp_stream(tcp);
            sshSession.handshake()?;
            sshSession.userauth_pubkey_file(user, None, private_key_path, None)?;
            if !sshSession.authenticated() {
                return Err("authentication failed".into());
            }
            sshSession.set_keepalive(true, 30);
            Ok(Self { session: sshSession })
        }

        pub fn runCommand(&mut self,
                          command: &str,
                          sudo: bool) -> Result<CommandResult>
        {
            let mut channel: Channel = self.session.channel_session()?;
            // TODO: Refactor this
            if (sudo) {
                let sudo_cmd: String = format!("sudo -S -p '' {}", command);
                channel.exec(&sudo_cmd)?;
                channel.write_all(format!("{}\n", "test").as_bytes())?;
                channel.flush()?;
            } else {
                channel.exec(&command)?;
            }

            let mut result: CommandResult = CommandResult {
                stdout: String::new(),
                stderr: String::new(),
                exitCode: 0
            };

            channel.read_to_string(&mut result.stdout)?;
            channel.stderr().read_to_string(&mut result.stderr)?;

            channel.wait_close()?;
            result.exitCode = channel.exit_status()?;
            Ok(result)
        }

        pub fn is_alive(&self) -> bool {
            self.session.authenticated()
        }
    }

    #[async_trait]
    impl Manager for SshManager
    {
        type Type = Mutex<SshConnection>;
        type Error = DynError;

        async fn create(&self) -> std::result::Result<Self::Type, Self::Error>
        {
            println!("=====> Creating connection");
            let conn: SshConnection = SshConnection::establishSshConnection(
                &self.host, self.port, &self.user, &self.password, &self.private_key)?;
            Ok(Mutex::new(conn))
        }

        async fn recycle(&self,
                         conn: &mut Self::Type,
                         _: &Metrics) -> RecycleResult<Self::Error>
        {
            let conn: MutexGuard<SshConnection> = conn.lock().unwrap();
            if conn.is_alive() {
                Ok(())
            } else {
                Err(RecycleError::Message("SSH connection dead".into()))
            }
        }
    }

    async fn run_command(pool: &Pool<SshManager>, cmd: &str, sudo: bool) -> Result<CommandResult>
    {
        let connection: Object<SshManager> = pool.get().await
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(format!("{}", e)))?;
        let command: String = cmd.to_string();
        let result: CommandResult = tokio::task::spawn_blocking(move || {
            let mut conn: MutexGuard<SshConnection> = connection.lock().unwrap();
            conn.runCommand(&command, sudo)
        }).await??;
        Ok(result)
    }

    #[tokio::main]
    pub async fn execCommand(command: String) -> Result<()>
    {
        let manager: SshManager = SshManager::new("127.0.0.1", 22022, "test", "test",
                                                  env::current_dir().unwrap().join("resources/test_ssh_keys/id_ed25519")
        );
        let pool: Pool<SshManager, Object<SshManager>> = Pool::builder(manager).max_size(4).build()?;

        let result: CommandResult = run_command(&pool, &command, false).await?;
        println!("{}", result.stdout.trim());
        Ok(())
    }
}

fn main()
{
    let _ = ssh_pool_impl::execCommand("ls -lar".parse().unwrap());
}
