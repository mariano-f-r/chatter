use serde::Deserialize;
use serde_json;
use shuttle_runtime;
use std::fs;
use std::net;
use std::rc::Rc;
use ws::{listen, Error, ErrorKind, Handler, Handshake, Message, Request, Response};

#[derive(Deserialize, Debug)]
struct ChatMessage {
    username: String,
    time: String,
    content: String,
}

struct CustomService;

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CustomService {
    async fn bind(mut self, addr: net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let users: Rc<Vec<String>> = Rc::new(Vec::<String>::new());

        listen(addr, |output| ClientHandler { output, ip: None }).unwrap();
        Ok(())
    }
}

struct ClientHandler {
    output: ws::Sender,
    ip: Option<String>,
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
        let parsed_message: ChatMessage = match serde_json::from_str(msg_json) {
            Ok(chatmessage) => chatmessage,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Internal,
                    format!("Received arbitrary JSON: {}", e),
                ))
            }
        };
        let sendable_message = format!(
            "{} at {}: {}",
            parsed_message.username, parsed_message.time, parsed_message.content
        );
        self.output.broadcast(sendable_message)
    }
    fn on_open(&mut self, shake: Handshake) -> Result<(), ws::Error> {
        let address = shake.remote_addr()?;
        self.output.broadcast(format!(
            "{} has joined the chat",
            address.unwrap_or("unknown address".to_string())
        ))
    }
}

#[shuttle_runtime::main]
async fn init() -> Result<CustomService, shuttle_runtime::Error> {
    Ok(CustomService)
}
