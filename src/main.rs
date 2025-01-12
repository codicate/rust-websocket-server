mod rest;
mod ws;

use rest::Request;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use ws::{encode_message, ws_handshake, Frame, FrameKind};

struct Client {
    address: String,
    stream: TcpStream,
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
        clients.push(Client { address, stream });
    }
}

fn handle_clients(clients: &mut Vec<Client>) {
    let mut disconnected_clients = vec![];
    let mut messages: HashMap<String, String> = HashMap::new();

    for (index, client) in clients.iter_mut().enumerate() {
        let Ok(frame) = Frame::new(&client.stream) else {
            continue;
        };

        if frame.kind == FrameKind::Close {
            println!("Closing WebSocket connection: {}", client.address);
            client.stream.shutdown(std::net::Shutdown::Both).unwrap();
            disconnected_clients.push(index);
            continue;
        }

        let message = String::from_utf8(frame.payload).unwrap();
        println!("Received message from {}: {}", client.address, message);
        messages.insert(client.address.clone(), message);
    }

    for &index in disconnected_clients.iter().rev() {
        clients.remove(index);
    }

    for (address, message) in messages {
        let encoded_message = encode_message(message);
        for other in clients.iter_mut() {
            if other.address == address {
                continue;
            }

            other.stream.write_all(&encoded_message).unwrap();
        }
    }
}
