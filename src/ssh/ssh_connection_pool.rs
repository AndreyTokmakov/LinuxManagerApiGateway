use std::env;
use crate::ssh_connection_pool::ssh_connection_pool::{CommandResult, SshCommandRunner};

pub(crate) mod ssh_connection_pool
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

    pub struct CommandResult
    {
        pub stdout: String,
        pub stderr: String,
        pub exitCode: i32,
    }

    pub struct SshConnection {
        session: Session,
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
            
            println!("Running command: {}", command);

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

    struct SshManager
    {
        host: String,
        port: u16,
        user: String,
        password: String,
        private_key: PathBuf,
    }

    impl SshManager
    {
        fn new(host: &str,
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

    #[async_trait]
    impl Manager for SshManager
    {
        type Type = Mutex<SshConnection>;
        type Error = DynError;

        async fn create(&self) -> std::result::Result<Self::Type, Self::Error>
        {
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

    pub struct SshCommandRunner
    {
        pool: Pool<SshManager, Object<SshManager>>
    }

    impl SshCommandRunner
    {
        pub fn new(host: &str,
                   port: u16,
                   user: &str,
                   password: &str,
                   private_key_path: PathBuf) -> Self
        {
            let manager: SshManager = SshManager::new(host, port, user, password,private_key_path);
            let pool: Pool<SshManager, Object<SshManager>> = Pool::builder(manager)
                .max_size(4).build().expect("Failed to create pool");
            Self { pool }
        }

        async fn runCommand(&self, command: String, sudo: bool) -> Result<CommandResult>
        {
            let connection: Object<SshManager> = self.pool.get().await
                .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(format!("{}", e)))?;

            let result: CommandResult = tokio::task::spawn_blocking(move || {
                let mut conn: MutexGuard<SshConnection> = connection.lock().unwrap();
                conn.runCommand(&command, sudo)
            }).await??;
            Ok(result)
        }

        pub async fn execCommand(&self, cmd: &str, sudo: bool)-> Result<CommandResult>
        {
            let command: String = cmd.to_string();
            let result: CommandResult = self.runCommand(command, false).await?;
            Ok(result)
        }
    }
}

pub fn get_ssh_cmd_runner() -> SshCommandRunner
{
    SshCommandRunner::new(
        "127.0.0.1",
        22022,
        "test",
        "test",
        env::current_dir().unwrap().join("resources/test_ssh_keys/id_ed25519")
    )
}

pub async fn test_all() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let runner: SshCommandRunner = get_ssh_cmd_runner();
    let output: CommandResult = runner.execCommand("ls -lar", false).await?;
    println!("{}", output.stdout);
    Ok(())
}
