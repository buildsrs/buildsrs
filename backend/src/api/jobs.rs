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
}

impl Connection {
    async fn recv(&mut self) -> Result<ClientMessage, WebSocketError> {
        while let Some(message) = self.websocket.next().await {
            let message: SignedMessage<ClientMessage> = match message? {
                Message::Text(message) => serde_json::from_str(&message)?,
                _ => continue,
            };
            // TODO: verify
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
                ClientMessage::ChallengeResponse(response) => match challenge == response {
                    true => return Ok(()),
                    false => return Err(WebSocketError::ChallengeError),
                },
                _ => continue,
            }
        }
    }
}

impl Backend {
    pub async fn handle_jobs(&self, mut websocket: WebSocket) -> Result<(), WebSocketError> {
        let fingerprint = extract_fingerprint(&mut websocket).await?;
        let mut connection = Connection { websocket };
        connection.challenge().await?;

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
