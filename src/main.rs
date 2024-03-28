use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use uuid::{self, Uuid};

#[derive(Debug)]
struct User {
    address: SocketAddr,
    username: Option<String>,
    sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    uuid: uuid::Uuid,
}

impl User {
    fn new(
        address: SocketAddr,
        sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        uuid: Uuid,
    ) -> Self {
        User {
            address,
            username: None,
            sender,
            uuid,
        }
    }

    async fn send_chat_message(&mut self, msg: &ChatMessage) {
        let ser = serde_json::to_string(msg).expect("Should be able to serialize string to test");
        self.sender
            .send(Message::Text(ser))
            .await
            .expect("Should be able to send message via WebSocket through sink");
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum ChatMessage {
    Message {
        username: String,
        time: String,
        content: String,
    },
}
// The ChatMessage should encapsulate system messages as well

#[derive(Debug)]
enum ThreadMessages {
    NewUser(User),
    UserLeave(Uuid),
    UserMessage(ChatMessage),
    KickUser(Uuid),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = TcpListener::bind("127.0.0.1:8000").await?;
    let (tx, mut rx) = mpsc::channel(32);

    tokio::spawn(async move {
        let mut connections: Vec<User> = Vec::new();

        while let Some(message) = rx.recv().await {
            match message {
                ThreadMessages::NewUser(user) => {
                    println!("New user from {}", user.address);
                    connections.push(user);
                }
                ThreadMessages::UserLeave(unique_ident) => {
                    connections.retain(|user| user.uuid != unique_ident);
                }
                ThreadMessages::UserMessage(chat_message) => {
                    for user in &mut connections {
                        user.send_chat_message(&chat_message).await;
                    }
                }
                ThreadMessages::KickUser(_uuid) => {
                    println!("Need to implement");
                }
            }
        }
    });

    while let Ok((stream, address)) = socket.accept().await {
        let ws_stream = accept_async(stream)
            .await
            .expect("Should be able to connect");

        let new_transmitter = tx.clone();

        tokio::spawn(async move {
            let (sender, receiver) = ws_stream.split();
            let unique_ident = Uuid::new_v4();
            let user = User::new(address, sender, unique_ident);
            new_transmitter
                .send(ThreadMessages::NewUser(user))
                .await
                .expect("Should be able to send messages across threads");
            // .expect() is used because in the event of a failure to send a message across threads,
            // the whole architecture of the app breaks down anyways, so it is better to panic than
            // leave the app running nonfunctionally

            receiver
                .for_each(|message| async {
                    if let Ok(msg) = message {
                        if let Ok(msg_txt) = msg.to_text() {
                            match serde_json::from_str(msg_txt) {
                                Ok(chat_message) => {
                                    new_transmitter
                                        .send(ThreadMessages::UserMessage(chat_message))
                                        .await
                                        .expect("Should be able to send messages across threads");
                                }
                                Err(e) => {
                                    dbg!(e);
                                    new_transmitter
                                        .send(ThreadMessages::KickUser(unique_ident))
                                        .await
                                        .expect("Should be able to send messages across threads");
                                }
                            };
                        } else {
                        }
                    } else {
                        eprintln!("Could not read message.")
                    }
                })
                .await;

            new_transmitter
                .send(ThreadMessages::UserLeave(unique_ident))
                .await
                .expect("Should be able to send messages across threads");
            // same as above
        });
    }

    Ok(())
}
