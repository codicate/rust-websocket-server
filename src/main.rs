mod rest;
mod ws;

use rest::Request;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
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
            Err(_) => {} // No new connection, continue the loop
            Ok((stream, _)) => {
                stream.set_nonblocking(true).unwrap();
                handle_connection(stream, &mut clients);
            }
        }

        // handle_clients(&mut clients);
    }
}

fn handle_connection(mut stream: TcpStream, clients: &mut Vec<Client>) {
    let Ok(request) = Request::new(&stream) else {
        println!("Failed to read request");
        return;
    };
    println!("Received request: {} {}", request.method, request.path);

    if request.path == "/ws" {
        let response = ws_handshake(request);
        stream.write_all(response.to_string().as_bytes()).unwrap();

        let address = stream.peer_addr().unwrap().to_string();
        println!("New WebSocket connection: {}", address);
        clients.push(Client { address, stream });
    }
}

fn handle_clients(clients: &mut Vec<Client>) {
    for client in clients.iter_mut() {
        let Ok(frame) = Frame::new(&client.stream) else {
            println!("Failed to read frame");
            continue;
        };

        if frame.kind == FrameKind::Close {
            println!("Closing WebSocket connection: {}", client.address);
            client.stream.shutdown(std::net::Shutdown::Both).unwrap();
            return;
        }

        let message = String::from_utf8(frame.payload).unwrap();
        println!("Received message from {}: {}", client.address, message);

        let response = "Hi, how can I help you?";
        let response_bytes = encode_message(response);
        client.stream.write_all(&response_bytes).unwrap();
    }
}
