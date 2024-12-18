use crate::parsing::{http_request::HttpRequest, HttpMethod};
use std::collections::HashMap as Map;
use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use HttpMethod::*;

enum State {
    Method,
    Path,
    QueryKey,
    QueryValue,
    HeadersKey,
    HeadersValue,
    Body,
}
pub struct RequestParser<'a> {
    stream: &'a mut TcpStream,
    state: State,
    buf: Vec<u8>,
    content_length: usize,
    content_received: usize,
}

#[derive(Debug)]
pub enum ParseError {
    StreamError(Box<dyn Error>),
    MalformedRequest,
    InvalidMethod,
    ContentTooSmall,
    ContentTooLarge,
    GenericError,
}
impl<'a> RequestParser<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        RequestParser {
            stream,
            state: State::Method,
            buf: Vec::new(),
            content_length: 0,
            content_received: 0,
        }
    }

    pub fn parse(&mut self) -> Result<HttpRequest, ParseError> {
        if self.buf.capacity() == 0 {
            self.buf.resize(1024, 0);
        }
        let mut buf = BufReader::new(&mut self.stream);
        let mut first_line = String::new();
        buf.read_line(&mut first_line).map_err(|err| {
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                ParseError::MalformedRequest
            } else {
                ParseError::StreamError(Box::new(err))
            }
        })?;
        let mut parts = first_line.split_whitespace();
        let method_str = parts.next().ok_or(ParseError::MalformedRequest)?;
        let method = match method_str {
            "GET" => Get,
            "POST" => Post,
            "PUT" => Put,
            "DELETE" => Delete,
            "PATCH" => Patch,
            "HEAD" => Head,
            "OPTIONS" => Options,
            "TRACE" => Trace,
            "CONNECT" => Connect,
            _ => return Err(ParseError::InvalidMethod),
        };
        let full_path = parts.next().ok_or(ParseError::MalformedRequest)?;
        let query_idx = full_path.find('?');
        let path = if let Some(idx) = query_idx {
            full_path[..idx].to_string()
        } else {
            full_path.to_string()
        };
        let mut query = Map::new();
        if let Some(idx) = query_idx {
            let query_str = &full_path[idx + 1..];
            for pair in query_str.split('&') {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next().ok_or(ParseError::MalformedRequest)?;
                let value = parts.next().ok_or(ParseError::MalformedRequest)?;
                query.insert(key.to_string(), value.to_string());
            }
        }
        let mut headers = Map::new();
        loop {
            let mut line = String::new();
            buf.read_line(&mut line).map_err(|err| {
                if err.kind() == std::io::ErrorKind::UnexpectedEof {
                    ParseError::MalformedRequest
                } else {
                    ParseError::StreamError(Box::new(err))
                }
            })?;
            if line.trim().is_empty() {
                break;
            }
            let mut parts = line.splitn(2, ':');
            let key = parts.next().ok_or(ParseError::MalformedRequest)?;
            let mut value = parts.next().ok_or(ParseError::MalformedRequest)?;
            value = value.trim();
            headers.insert(key.to_string(), value.to_string());
        }

        let mut content_length = headers
            .get("Content-Length")
            .map(|value| value.parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        if content_length == 0 {
            let req = HttpRequest::new(method, path, query, headers, None);
            return Ok(req);
        }

        let mut body = vec![0; content_length];

        buf.read_exact(&mut body).map_err(|err| {
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                ParseError::ContentTooSmall
            } else {
                ParseError::StreamError(Box::new(err))
            }
        })?;
        Ok(HttpRequest::new(method, path, query, headers, Some(body)))
    }
}
