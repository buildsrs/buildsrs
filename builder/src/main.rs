use anyhow::Result;
use buildsrs_protocol::*;
use clap::Parser;
use duration_string::DurationString;
use futures::{SinkExt, StreamExt};
use ssh_key::{HashAlg, PrivateKey};
use std::path::PathBuf;
use tokio::{
    net::TcpStream,
    sync::mpsc::{channel, Receiver, Sender},
    time::timeout,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::*;
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct Options {
    #[clap(long, short, env)]
    pub private_key_file: PathBuf,

    #[clap(long, short, env)]
    pub websocket: Url,

    #[clap(long, env, default_value = "1m")]
    pub timeout_connect: DurationString,

    #[clap(long, env, default_value = "1m")]
    pub timeout_authenticate: DurationString,
}

pub struct Connection {
    private_key: PrivateKey,
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    receiver: Receiver<()>,
    sender: Sender<()>,
}

impl Connection {
    pub async fn connect(private_key: PrivateKey, url: &Url) -> Result<Self> {
        let (websocket, _) = connect_async(url.as_str()).await?;
        let (sender, receiver) = channel(16);
        Ok(Self {
            private_key,
            websocket,
            receiver,
            sender,
        })
    }

    pub async fn send(&mut self, message: ClientMessage) -> Result<()> {
        let signed = SignedMessage::new(&self.private_key, message)?;
        let json = serde_json::to_string(&signed)?;
        self.websocket.send(Message::Text(json)).await?;
        Ok(())
    }

    pub async fn authenticate(&mut self) -> Result<()> {
        let fingerprint = self.private_key.public_key().fingerprint(HashAlg::Sha512);
        self.send(ClientMessage::Hello(fingerprint)).await?;
        let challenge = loop {
            if let Some(message) = self.websocket.next().await {
                let message: ServerMessage = match message? {
                    Message::Text(text) => serde_json::from_str(&text)?,
                    other => continue,
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
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::parse();
    tracing_subscriber::fmt::init();

    debug!("Reading private key from {:?}", options.private_key_file);
    let private_key = PrivateKey::read_openssh_file(&options.private_key_file)?;
    info!(
        "Read private key {}",
        private_key.fingerprint(HashAlg::Sha512)
    );

    debug!(
        "Connecting to WebSocket, timeout {}",
        options.timeout_connect
    );
    let mut connection = timeout(
        options.timeout_connect.into(),
        Connection::connect(private_key, &options.websocket),
    )
    .await??;
    info!("Connected to {}", options.websocket);

    debug!(
        "Authenticating with WebSocket, timeout {}",
        options.timeout_authenticate
    );
    timeout(
        options.timeout_authenticate.into(),
        connection.authenticate(),
    )
    .await??;
    info!("Authenticated with WebSocket");

    Ok(())
}
