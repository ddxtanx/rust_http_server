use std::collections::HashMap as Map;
use std::io::Write;
use std::net::TcpStream;

pub struct HttpResponse {
    status: u16,
    headers: Map<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: u16, headers: Map<String, String>, body: Vec<u8>) -> Self {
        HttpResponse {
            status,
            headers,
            body,
        }
    }

    pub fn write_to_stream(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        let status_line = format!("HTTP/1.1 {}\r\n", self.status);
        stream.write_all(status_line.as_bytes())?;
        for (key, value) in &self.headers {
            let header_line = format!("{}: {}\r\n", key, value);
            stream.write_all(header_line.as_bytes())?;
        }
        stream.write_all(b"\r\n")?;
        stream.write_all(&self.body)?;
        Ok(())
    }

    pub fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    pub fn set_status(&mut self, status: u16) {
        self.status = status;
    }

    pub fn get_body(&self) -> &Vec<u8> {
        &self.body
    }

    pub fn get_status(&self) -> u16 {
        self.status
    }

    pub fn get_headers(&self) -> &Map<String, String> {
        &self.headers
    }
}
