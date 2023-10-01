use serde::Deserialize;
use serde_json;
use std::fs;
use std::rc::Rc;
use ws::{listen, Error, ErrorKind, Handler, Handshake, Message, Request, Response, Result};

#[derive(Deserialize, Debug)]
struct ChatMessage {
    username: String,
    time: String,
    content: String,
}

struct Server {
    output: ws::Sender,
    ip: Option<String>,
}

impl Handler for Server {
    fn on_request(&mut self, req: &Request) -> Result<Response> {
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
    fn on_message(&mut self, msg: Message) -> Result<()> {
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
        dbg!("{}", &parsed_message);
        let sendable_message = format!(
            "{} at {}: {}",
            parsed_message.username, parsed_message.time, parsed_message.content
        );
        self.output.broadcast(sendable_message)
    }
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        let address = shake.remote_addr()?;
        self.output.broadcast(format!(
            "{} has joined the chat",
            address.unwrap_or("unknown address".to_string())
        ))
    }
}

fn main() {
    let users: Rc<Vec<String>> = Rc::new(Vec::<String>::new());
    let host = "192.168.137.33:8000";
    println!("Running on http://{}", host);
    listen(host, |output| Server { output, ip: None }).unwrap();
}
