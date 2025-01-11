use std::io::Read;
use std::{collections::HashMap, net::TcpStream};

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
}

impl Request {
    pub fn new(mut stream: &TcpStream) -> std::io::Result<Self> {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer).to_owned();
        let mut lines = request.lines();

        let mut info = lines.next().unwrap().split_whitespace();
        let method = info.next().unwrap().to_string();
        let path = info.next().unwrap().to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if line.trim().is_empty() {
                break; // End of headers
            }
            if let Some((key, value)) = line.split_once(":") {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(Request {
            method,
            path,
            headers,
        })
    }
}

pub struct Response {
    pub status_code: u16,
    pub status: String,
    pub headers: HashMap<String, String>,
}

impl Response {
    pub fn new(status_code: u16, status: &str) -> Self {
        Response {
            status_code,
            status: status.to_string(),
            headers: HashMap::new(),
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn to_string(&self) -> String {
        let mut response = format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status);

        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        // End the headers section with a blank line
        response.push_str("\r\n");
        response
    }
}
