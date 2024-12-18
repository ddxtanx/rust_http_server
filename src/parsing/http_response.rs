use crate::helpers::path_to_mimetype;
use json::json::JSON;
use std::collections::HashMap as Map;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct HttpResponse<'a> {
    status: u16,
    headers: Map<String, String>,
    content_type: Option<&'a str>,
    body: Vec<u8>,
}

impl<'a> HttpResponse<'a> {
    pub fn new(
        status: u16,
        headers: Map<String, String>,
        content_type: Option<&'a str>,
        body: Vec<u8>,
    ) -> Self {
        HttpResponse {
            status,
            headers,
            content_type,
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
        let content_length = format!("Content-Length: {}\r\n", self.body.len());
        stream.write_all(content_length.as_bytes())?;
        let content_type = match self.content_type {
            Some(content_type) => format!("Content-Type: {}\r\n", content_type),
            None => String::from("text/plain"),
        };
        stream.write_all(content_type.as_bytes())?;
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

    pub fn set_content_type(&mut self, content_type: &'a str) {
        self.content_type = Some(content_type);
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

    pub fn get_content_type(&self) -> &Option<&'a str> {
        &self.content_type
    }

    pub fn write_from_json(stored_response: &mut Self, json: &JSON) {
        let json_string = json.to_string();
        stored_response.set_body(json_string.as_bytes().to_vec());
        stored_response.set_content_type("application/json");
    }

    pub fn write_from_string(stored_response: &mut Self, string: &str) {
        stored_response.set_body(string.as_bytes().to_vec());
        stored_response.set_content_type("text/plain");
    }

    pub fn write_from_file(stored_response: &mut Self, path: &str) -> Result<(), std::io::Error> {
        let os_path = std::path::Path::new(path);
        if !os_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ));
        }
        let mut file = std::fs::File::open(os_path)?;
        let size = file.metadata()?.len() as usize;
        let mut buffer = vec![0; size];
        file.read_exact(&mut buffer)?;
        stored_response.set_body(buffer);
        stored_response.set_content_type(path_to_mimetype(os_path));
        Ok(())
    }
}
