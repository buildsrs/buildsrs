use crate::Backend;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use buildsrs_database::{entity::Builder, Error as DatabaseError};
use buildsrs_protocol::{ssh_key::Fingerprint, *};
use futures::StreamExt;

#[derive(thiserror::Error, Debug)]
pub enum WebSocketError {
    #[error("Missing Hello message")]
    MissingHello,
    #[error("Challenge incorrect")]
    ChallengeError,
    #[error("Stream is closed")]
    StreamClosed,
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Signature(#[from] SignatureError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

async fn extract_fingerprint(socket: &mut WebSocket) -> Result<Fingerprint, WebSocketError> {
    while let Some(message) = socket.next().await {
        let message: SignedMessage<ClientMessage> = match message? {
            Message::Text(message) => serde_json::from_str(&message)?,
            _ => continue,
        };
        match message.message {
            ClientMessage::Hello(fingerprint) => return Ok(fingerprint),
            _ => continue,
        }
    }

    Err(WebSocketError::MissingHello)
}

struct Connection {
    websocket: WebSocket,
    builder: Builder,
}

impl Connection {
    async fn recv(&mut self) -> Result<ClientMessage, WebSocketError> {
        while let Some(message) = self.websocket.next().await {
            let message: SignedMessage<ClientMessage> = match message? {
                Message::Text(message) => serde_json::from_str(&message)?,
                _ => continue,
            };
            message.verify(&self.builder.public_key)?;
            return Ok(message.message);
        }
        Err(WebSocketError::StreamClosed)
    }

    async fn send(&mut self, message: ServerMessage) -> Result<(), WebSocketError> {
        self.websocket
            .send(Message::Text(serde_json::to_string(&message)?))
            .await?;
        Ok(())
    }

    async fn challenge(&mut self) -> Result<(), WebSocketError> {
        let challenge = [0xab].to_vec();
        self.send(ServerMessage::ChallengeRequest(challenge.clone().into()))
            .await?;
        loop {
            let message = self.recv().await?;
            match message {
                ClientMessage::ChallengeResponse(response) => {
                    return if challenge == response {
                        Ok(())
                    } else {
                        Err(WebSocketError::ChallengeError)
                    }
                }
                _ => continue,
            }
        }
    }

    async fn handle_job_request(
        &mut self,
        _request: &JobRequest,
    ) -> Result<ServerMessage, WebSocketError> {
        todo!()
    }

    async fn handle(&mut self) -> Result<(), WebSocketError> {
        loop {
            let response = match self.recv().await? {
                ClientMessage::Hello(_) | ClientMessage::ChallengeResponse(_) => break,
                ClientMessage::JobRequest(request) => self.handle_job_request(&request).await?,
            };
            self.send(response).await?;
        }
        Ok(())
    }
}

impl Backend {
    /// Handle jobs websocket connection.
    pub async fn handle_jobs(&self, mut websocket: WebSocket) -> Result<(), WebSocketError> {
        let fingerprint = extract_fingerprint(&mut websocket).await?;
        let uuid = self
            .database()
            .builder_lookup(&fingerprint.to_string())
            .await?;
        let builder = self.database().builder_get(uuid).await?;
        let mut connection = Connection { websocket, builder };
        connection.challenge().await?;
        connection.handle().await?;
        Ok(())
    }
}

async fn jobs_websocket(State(backend): State<Backend>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| {
        let backend = backend.clone();
        async move {
            match backend.handle_jobs(socket).await {
                Ok(()) => {}
                Err(error) => println!("{error}"),
            }
        }
    })
}

pub fn routes() -> Router<Backend> {
    Router::new().route("/jobs", get(jobs_websocket))
}
