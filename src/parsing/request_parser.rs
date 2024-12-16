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
        let mut line_iter = BufReader::new(&mut self.stream).lines();
        let first_line = line_iter
            .next()
            .ok_or(ParseError::GenericError)?
            .map_err(|err| ParseError::StreamError(Box::new(err)))?;
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
            let line = line_iter
                .next()
                .ok_or(ParseError::MalformedRequest)?
                .map_err(|err| ParseError::StreamError(Box::new(err)))?;
            if line.is_empty() {
                break;
            }
            let mut parts = line.splitn(2, ':');
            let key = parts.next().ok_or(ParseError::MalformedRequest)?;
            let mut value = parts.next().ok_or(ParseError::MalformedRequest)?;
            value = value.trim();
            headers.insert(key.to_string(), value.to_string());
        }

        let content_length = headers
            .get("Content-Length")
            .map(|value| value.parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        if content_length == 0 {
            let req = HttpRequest::new(method, path, query, headers, None);
            return Ok(req);
        }

        let mut body = vec![0; content_length];
        let bytes_read = self
            .stream
            .read(&mut body)
            .map_err(|err| ParseError::StreamError(Box::new(err)))?;
        if bytes_read < content_length {
            return Err(ParseError::ContentTooSmall);
        }
        let dummy_buffer = &mut [0; 1];
        let new_bytes_read = self
            .stream
            .read(dummy_buffer)
            .map_err(|err| ParseError::StreamError(Box::new(err)))?;
        if new_bytes_read > 0 {
            return Err(ParseError::ContentTooLarge);
        }

        Ok(HttpRequest::new(method, path, query, headers, Some(body)))
    }
}
