use serde::{Deserialize, Serialize};
use std::fs;
use std::net;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use ws::{listen, Handler, Handshake, Message, Request, Response};

// Anything received in some way from one client and/or broadcasted to the rest should be
// represented here
#[derive(Serialize, Deserialize, Debug)]
enum ChatEvents {
    // Contains the new user count within
    UserCountChange(u32),
    SystemMessage(String),
    ChatMessage {
        username: String,
        time: String,
        content: String,
    },
    TypingEvent {
        username: String,
        is_starting: bool,
    },
}

struct CustomService {
    user_count: Arc<AtomicU32>,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CustomService {
    async fn bind(mut self, addr: net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        listen(addr, move |output| {
            let user_count_ref = self.user_count.clone(); // Clone only if necessary
            ClientHandler {
                output,
                user_count_ref,
            }
        })
        .map_err(|err| shuttle_runtime::Error::from(err))?;
        
        Ok(())
    }
}

struct ClientHandler {
    output: ws::Sender,
    user_count_ref: Arc<AtomicU32>,
}

impl Handler for ClientHandler {
    fn on_request(&mut self, req: &Request) -> Result<Response, ws::Error> {
        println!("{} request to {:?}", req.method(), req.resource());
        match req.resource() {
            "/ws" => Response::from_request(req),
            "/" => Ok(Response::new(
                200,
                "OK",
                fs::read("static/home.html").expect("Should be able to read file"),
            )),
            "/static/main.js" => Ok(Response::new(
                200,
                "OK",
                fs::read("static/main.js").expect("Should be able to read file"),
            )),
            "/favicon.ico" => Ok(Response::new(
                200,
                "OK",
                fs::read("static/favicon.ico").expect("Should be able to read file"),
            )),
            _ => Ok(Response::new(
                404,
                "Not Found",
                b"404 - Resource Not Found".to_vec(),
            )),
        }
    }
    fn on_message(&mut self, msg: Message) -> Result<(), ws::Error> {
        let msg_json = msg.as_text()?;
        let parsed_message: ChatEvents = match serde_json::from_str(msg_json) {
            Ok(chatmessage) => chatmessage,
            Err(e) => {
                self.output.send(
                    serde_json::to_string(&ChatEvents::SystemMessage(String::from(
                        "You have been kicked for sending arbitrary JSON to the server.",
                    )))
                    .expect("Should have been able to serialize this message"),
                )?;
                self.output.close(ws::CloseCode::Invalid)?;
                println!("Received arbitrary JSON {:#?}", e);
                return Ok(());
            }
        };
        // There is certainly a better way to do this, but in short, all this does is match based
        // on if its a valid enum variant, and if so, just broadcast the original message instead
        // of re-serializing it. This way is type-safe to my knowledge.
        match parsed_message {
            ChatEvents::ChatMessage {
                username: _,
                time: _,
                content: _,
            } => self.output.broadcast(msg_json)?,
            ChatEvents::TypingEvent {
                username: _,
                is_starting: _,
            } => self.output.broadcast(msg_json)?,
            event => println!("{:#?}", event),
        }
        Ok(())
    }
    fn on_open(&mut self, _: Handshake) -> Result<(), ws::Error> {
        // This line isn't particularly pleasant, but it minimizes accesses to the atomically
        // reference counted usercount
        let count = self.user_count_ref.fetch_add(1, Ordering::SeqCst) + 1;
        let sendable_message = ChatEvents::UserCountChange(count);
        let sendable_message: String = match serde_json::to_string(&sendable_message) {
            Ok(msg) => msg,
            Err(e) => {
                return Err(ws::Error::new(
                    ws::ErrorKind::Internal,
                    format!("Failed to serialize data: {}", e),
                ))
            }
        };
        self.output.broadcast(sendable_message)?;
        self.output.broadcast(
            serde_json::to_string(&ChatEvents::SystemMessage(String::from(
                "Someone has joined",
            )))
            .expect("Should be able to serialize message"),
        )
    }
    fn on_close(&mut self, _: ws::CloseCode, _: &str) {
        // Same as before
        let count = self.user_count_ref.fetch_sub(1, Ordering::SeqCst) - 1;
        let sendable_message = ChatEvents::UserCountChange(count);
        let sendable_message: String =
            serde_json::to_string(&sendable_message).expect("Should be able to serialize data");
        self.output
            .broadcast(sendable_message)
            .expect("Should be able to send message");
        self.output
            .broadcast(
                serde_json::to_string(&ChatEvents::SystemMessage(String::from(
                    "Somebody has left",
                )))
                .expect("Should be able to serialize message"),
            )
            .expect("Should be able to send goodbye message")
    }
}

#[shuttle_runtime::main]
async fn init() -> Result<CustomService, shuttle_runtime::Error> {
    Ok(CustomService {
        user_count: Arc::new(AtomicU32::new(0)),
    })
}
