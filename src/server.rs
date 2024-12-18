use crate::helpers::path_to_mimetype;
use crate::parsing::http_request::HttpRequest;
use crate::parsing::http_response::HttpResponse;
use crate::parsing::request_parser::RequestParser;
use std::collections::HashMap as Map;
use std::io::{Read, Result};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub struct Server<'a> {
    socket: TcpListener,
    handlers: Map<String, Box<dyn Fn(&HttpRequest, &mut HttpResponse) + 'a>>,
    static_asset_folder: Option<&'static str>,
}

impl<'a> Server<'a> {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let socket = TcpListener::bind(addr)?;
        Ok(Server {
            socket,
            handlers: Map::new(),
            static_asset_folder: None,
        })
    }

    pub fn set_static_asset_folder(&mut self, folder: &'static str) {
        self.static_asset_folder = Some(folder);
    }

    fn handle_static(&self, stream: &mut TcpStream, request: &HttpRequest) -> Result<bool> {
        if self.static_asset_folder.is_none() {
            return Ok(false);
        }

        let folder = self.static_asset_folder.as_ref().unwrap();
        let path = request.get_path();
        let mut path_str = String::from(path);

        if path_str.ends_with('/') {
            path_str.push_str("index.html");
        }

        let mut resp = HttpResponse::new(200, Map::new(), None, Vec::new());
        let read_result = HttpResponse::write_from_file(&mut resp, path_str.as_str());
        if read_result.is_err() {
            return Ok(false);
        }
        resp.write_to_stream(stream)?;
        Ok(true)
    }

    fn handle_request(&self, stream: &mut TcpStream, request: &HttpRequest) -> Result<()> {
        let was_static = self.handle_static(stream, request)?;
        if was_static {
            return Ok(());
        }

        let path = request.get_path();
        let handler = self.handlers.get(path);
        let mut response = HttpResponse::new(0, Map::new(), None, Vec::new());
        match handler {
            Some(handler) => {
                handler(request, &mut response);
                response.write_to_stream(stream)?;
            }
            None => {
                response.write_to_stream(stream)?;
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        for stream in self.socket.incoming() {
            let mut stream = stream?;
            let mut parser = RequestParser::new(&mut stream);
            let request = parser.parse();
            if request.is_err() {
                println!("Failed to parse request");
                continue;
            }
            let request = request.unwrap();
            self.handle_request(&mut stream, &request)?;
            println!("Connection established!");
        }
        Ok(())
    }

    pub fn add_handler<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&HttpRequest, &mut HttpResponse) + 'a,
    {
        self.handlers.insert(path.to_string(), Box::new(handler));
    }
}
