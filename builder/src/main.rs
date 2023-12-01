#![allow(missing_docs)]
use anyhow::Result;
use buildsrs_builder::StrategyOptions;
use buildsrs_protocol::*;
use clap::Parser;
use duration_string::DurationString;
use futures::{SinkExt, StreamExt};
use reqwest::Client;
use ssh_key::{HashAlg, PrivateKey};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    net::TcpStream,
    select,
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinSet,
    time::timeout,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::*;
use url::Url;

static BUILDER_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Default [`WebSocket`] endpoint.
const DEFAULT_WEBSOCKET: &str = "wss://api.builds.rs/api/v1/jobs";

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(flatten)]
    pub strategy: StrategyOptions,

    /// Path to SSH private key.
    ///
    /// SSH private key is used for authentication and for artifact signing.
    #[clap(long, short, env)]
    pub private_key_file: PathBuf,

    /// Target this builder will build.
    #[clap(long, env, default_value = "x86_86-unknown-linux-gnu")]
    pub target: String,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug, Clone)]
pub enum Command {
    Build(BuildCommand),
    Connect(ConnectCommand),
}

/// Build a single crate.
#[derive(Parser, Debug, Clone)]
pub struct BuildCommand {
    /// Crate Url to build
    #[clap(default_value = "https://static.crates.io/crates/ripgrep/ripgrep-13.0.0.crate")]
    pub source: Url,

    /// Checksum to build
    #[clap(
        long,
        default_value = "f37c9d2c2bc7e00bd2653e13771397b94e452583da9b9494eabef627618d64bf"
    )]
    pub checksum: Option<String>,
}

/// Connect to backend to service jobs.
#[derive(Parser, Debug, Clone)]
pub struct ConnectCommand {
    /// WebSocket endpoint to connect to.
    #[clap(long, short, env, default_value = DEFAULT_WEBSOCKET)]
    pub websocket: Url,

    /// Timeout for connection to backend.
    #[clap(long, env, default_value = "1m")]
    pub timeout_connect: DurationString,

    /// Timeout for authentication with backend.
    #[clap(long, env, default_value = "1m")]
    pub timeout_authenticate: DurationString,

    /// Job many jobs to run in parallel.
    #[clap(long, env, default_value = "1")]
    pub parallel: usize,
}

/// [`WebSocketStream`] connection type alias.
type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub enum Event {
    Build(String),
}

pub struct Connection {
    /// Private key, used for authentication and artifact signing.
    private_key: PrivateKey,
    /// WebSocket connection.
    websocket: WebSocket,
    /// List of currently running jobs.
    tasks: JoinSet<()>,
    /// Event receiver.
    receiver: Receiver<Event>,
    /// Backlog of jobs, if there are more than can fit.
    backlog: Vec<Job>,
    /// Sender of events.
    sender: Sender<Event>,
}

impl Connection {
    /// Connect to [`WebSocket`] endpoint.
    pub async fn connect(private_key: PrivateKey, url: &Url) -> Result<Self> {
        let (websocket, _) = connect_async(url.as_str()).await?;
        Ok(Self::new(websocket, private_key))
    }

    /// Create new connection.
    pub fn new(websocket: WebSocket, private_key: PrivateKey) -> Self {
        let (sender, receiver) = channel(16);
        Self {
            private_key,
            websocket,
            sender,
            receiver,
            tasks: Default::default(),
            backlog: Default::default(),
        }
    }

    /// Send a signed [`ClientMessage`].
    pub async fn send(&mut self, message: ClientMessage) -> Result<()> {
        let signed = SignedMessage::new(&self.private_key, message)?;
        let json = serde_json::to_string(&signed)?;
        self.websocket.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Receive a [`ServerMessage`].
    pub async fn recv(websocket: &mut WebSocket) -> Result<ServerMessage> {
        loop {
            match websocket.next().await {
                Some(Ok(Message::Text(_text))) => todo!(),
                _ => todo!(),
            }
        }
    }

    /// Authenticate to server.
    pub async fn authenticate(&mut self) -> Result<()> {
        let fingerprint = self.private_key.public_key().fingerprint(HashAlg::Sha512);
        self.send(ClientMessage::Hello(fingerprint)).await?;
        let challenge = loop {
            if let Some(message) = self.websocket.next().await {
                let message: ServerMessage = match message? {
                    Message::Text(text) => serde_json::from_str(&text)?,
                    _other => continue,
                };
                match message {
                    ServerMessage::ChallengeRequest(challenge) => break challenge,
                    _ => continue,
                }
            }
        };
        self.send(ClientMessage::ChallengeResponse(challenge))
            .await?;
        Ok(())
    }

    /// Synchronize tasks with server.
    pub async fn tasks_sync(&mut self) -> Result<()> {
        // TODO: implement tasks sync
        Ok(())
    }

    /// Handle a single iteration.
    pub async fn handle_iter(&mut self) -> Result<()> {
        select! {
            message = Self::recv(&mut self.websocket) => self.handle_message(message?),
            _result = self.tasks.join_next() => self.handle_done().await?,
            _event = self.receiver.recv() => {},
        }
        Ok(())
    }

    /// Handle messages and events.
    pub async fn handle(&mut self) -> Result<()> {
        loop {
            self.handle_iter().await?;
        }
    }

    async fn handle_done(&mut self) -> Result<()> {
        if let Some(job) = self.backlog.pop() {
            let sender = self.sender.clone();
            self.tasks.spawn(Self::job(job, sender));
        }
        Ok(())
    }

    fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::JobList(jobs) => {
                for job in jobs {
                    self.handle_job(job);
                }
            }
            ServerMessage::JobResponse(job) => self.handle_job(job),
            ServerMessage::ChallengeRequest(_) => unreachable!(),
        }
    }

    fn handle_job(&mut self, job: Job) {
        if self.tasks.len() > 8 {
            self.backlog.push(job);
        } else {
            let sender = self.sender.clone();
            self.tasks.spawn(Self::job(job, sender));
        }
    }

    pub async fn job(_job: Job, _sender: Sender<Event>) {}
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::parse();
    tracing_subscriber::fmt::init();

    let client = Client::builder().user_agent(BUILDER_USER_AGENT).build()?;

    debug!("Reading private key from {:?}", options.private_key_file);
    let private_key = PrivateKey::read_openssh_file(&options.private_key_file)?;
    info!(
        "Read private key {}",
        private_key.fingerprint(HashAlg::Sha512)
    );

    match options.command {
        Command::Connect(options) => {
            debug!("Connecting to WebSocket");
            let mut connection = timeout(
                options.timeout_connect.into(),
                Connection::connect(private_key, &options.websocket),
            )
            .await??;
            info!("Connected to {}", options.websocket);

            debug!("Authenticating with WebSocket",);
            timeout(
                options.timeout_authenticate.into(),
                connection.authenticate(),
            )
            .await??;
            info!("Authenticated with WebSocket");

            debug!("Synchronizing task list");
            connection.tasks_sync().await?;

            debug!("Handling events");
            connection.handle().await?;
        }
        Command::Build(build) => {
            let strategy = options.strategy.build().await?;
            let dir = TempDir::new()?;
            println!("dir is {dir:?}");

            let crate_file = client.get(build.source.as_str()).send().await?;
            let mut stream = crate_file.bytes_stream();
            let download_crate = dir.path().join("download.crate");
            let download_folder = dir.path().join("output");

            std::mem::forget(dir);
            let mut file = File::create(&download_crate).await?;
            while let Some(item) = stream.next().await {
                file.write_all(&item?).await?;
            }
            file.flush().await?;

            println!("Downloaded crate");

            let download_folder_clone = download_folder.clone();
            tokio::spawn(async move {
                use flate2::read::GzDecoder;
                use tar::Archive;
                let file = std::fs::File::open(&download_crate)?;
                std::fs::create_dir(&download_folder)?;
                let tar = GzDecoder::new(file);
                let mut archive = Archive::new(tar);
                archive.unpack(download_folder)?;
                Ok(()) as Result<()>
            })
            .await??;

            println!("Extracted crate");

            // find crate subfolder
            let mut download_folder = tokio::fs::read_dir(&download_folder_clone).await?;
            let download_folder = download_folder.next_entry().await?.unwrap().path();

            let builder = strategy.builder_from_path(&download_folder).await?;
            builder.metadata().await?;
        }
    }

    Ok(())
}
