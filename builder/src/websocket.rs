use anyhow::Result;
use buildsrs_protocol::*;
use futures::{SinkExt, StreamExt};
use ssh_key::{HashAlg, PrivateKey};
use std::time::Duration;
use tokio::{
    net::TcpStream,
    select,
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinSet,
    time::{interval, Interval},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::*;
use url::Url;

/// [`WebSocketStream`] connection type alias.
type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[allow(dead_code)]
pub enum Event {
    Build(String),
}

/// `WebSocket` connection to receive jobs
pub struct Connection {
    poll_timer: Interval,
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
    /// Connect to `WebSocket` endpoint.
    pub async fn connect(private_key: PrivateKey, url: &Url) -> Result<Self> {
        let (websocket, _) = connect_async(url.as_str()).await?;
        Ok(Self::new(websocket, private_key))
    }

    /// Create new connection.
    pub fn new(websocket: WebSocket, private_key: PrivateKey) -> Self {
        let (sender, receiver) = channel(16);
        Self {
            poll_timer: interval(Duration::from_secs(1)),
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
        if self.tasks.len() < 4 {
            info!("Requesting another task");
            self.send(ClientMessage::JobRequest(JobRequest {
                target: "generic".to_string(),
            }))
            .await?;
        }
        Ok(())
    }

    /// Handle a single iteration.
    pub async fn handle_iter(&mut self) -> Result<()> {
        select! {
            _tick = self.poll_timer.tick() => self.tasks_sync().await?,
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

    #[allow(clippy::no_effect_underscore_binding)]
    async fn job(_job: Job, _sender: Sender<Event>) {
        todo!()
    }
}
