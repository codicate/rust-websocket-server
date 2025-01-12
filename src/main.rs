mod rest;
mod ws;

use rest::Request;
use serde_json::{json, Value};
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use ws::{encode_message, ws_handshake, Frame, FrameKind};

struct Client {
    username: String,
    address: String,
    stream: TcpStream,
}

impl Client {
    fn new(address: String, stream: TcpStream) -> Self {
        Client {
            username: String::new(),
            address,
            stream,
        }
    }
}

fn main() {
    let mut clients: Vec<Client> = vec![];
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    listener.set_nonblocking(true).unwrap();
    println!("Server started at 127.0.0.1:8080");

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true).unwrap();
                handle_connection(stream, &mut clients);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }

        handle_clients(&mut clients);
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn handle_connection(mut stream: TcpStream, clients: &mut Vec<Client>) {
    let Ok(request) = Request::new(&stream) else {
        return;
    };

    if request.path == "/ws" {
        let response = ws_handshake(request);
        stream.write_all(response.to_string().as_bytes()).unwrap();

        let address = stream.peer_addr().unwrap().to_string();
        println!("New WebSocket connection: {}", address);

        let client = Client::new(address, stream);
        clients.push(client);
    }
}

fn handle_clients(clients: &mut Vec<Client>) {
    let mut disconnected_clients = vec![];
    let mut messages = vec![];

    for (index, client) in clients.iter_mut().enumerate() {
        let Ok(frame) = Frame::new(&client.stream) else {
            continue;
        };

        if frame.kind == FrameKind::Close {
            println!("Closing WebSocket connection: {}", client.address);
            client.stream.shutdown(std::net::Shutdown::Both).unwrap();
            disconnected_clients.push(index);
            add_message(
                &mut messages,
                "System".to_string(),
                format!("{} left the chat", client.username),
            );
            continue;
        }

        let data: Value = serde_json::from_slice(&frame.payload).unwrap();
        println!("Received data from {}: {}", client.address, data);

        if let Some(username) = data.get("username") {
            client.username = username.as_str().unwrap().to_string();
            add_message(
                &mut messages,
                "System".to_string(),
                format!("{} joined the chat", client.username),
            );
        }

        if let Some(message) = data.get("message") {
            add_message(
                &mut messages,
                client.username.clone(),
                message.as_str().unwrap().to_string(),
            );
        }
    }

    for &index in disconnected_clients.iter().rev() {
        clients.remove(index);
    }

    for message in messages {
        let encoded_message = encode_message(message);
        for client in clients.iter_mut() {
            client.stream.write_all(&encoded_message).unwrap();
        }
    }
}

fn add_message(messages: &mut Vec<String>, username: String, message: String) {
    let data = json!({ "username": username, "message": message });
    messages.push(data.to_string());
}
