use std::{collections::HashMap as Map, str::FromStr};

use json::{json::JSON, parsing::JSONError};

use crate::parsing::HttpMethod;

use super::http_response::HttpResponse;

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    query: Map<String, String>,
    headers: Map<String, String>,
    body: Option<Vec<u8>>,
}

pub enum RequestError {
    InvalidContentType,
    JSONError(JSONError),
}

impl<'a> From<RequestError> for HttpResponse<'a> {
    fn from(err: RequestError) -> HttpResponse<'a> {
        match err {
            RequestError::InvalidContentType => {
                let mut resp = HttpResponse::new(400, Map::new(), None, Vec::new());
                HttpResponse::write_from_string(&mut resp, "Invalid Content-Type");
                resp.set_status(400);
                resp
            }
            RequestError::JSONError(json_err) => {
                let mut resp = HttpResponse::new(400, Map::new(), None, Vec::new());
                HttpResponse::write_from_string(&mut resp, &format!("JSON Error: {}", json_err));
                resp.set_status(400);
                resp
            }
        }
    }
}

impl HttpRequest {
    pub fn get_method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_query(&self) -> &Map<String, String> {
        &self.query
    }

    pub fn get_headers(&self) -> &Map<String, String> {
        &self.headers
    }

    pub fn get_body(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }

    pub fn new(
        method: HttpMethod,
        path: String,
        query: Map<String, String>,
        headers: Map<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        HttpRequest {
            method,
            path,
            query,
            headers,
            body,
        }
    }

    pub fn json(&self) -> Result<JSON, RequestError> {
        let content_type = self.headers.get("Content-Type");
        if content_type == Some(&"application/json".to_string()) {
            let body_str = String::from_utf8_lossy(self.body.as_ref().unwrap());
            Ok(JSON::from_str(&body_str).map_err(RequestError::JSONError)?)
        } else {
            Err(RequestError::InvalidContentType)
        }
    }

    pub fn form(&self) -> Result<Map<String, String>, RequestError> {
        let content_type = self.headers.get("Content-Type");
        if content_type == Some(&"application/x-www-form-urlencoded".to_string()) {
            let body_str = String::from_utf8_lossy(self.body.as_ref().unwrap());
            let mut form = Map::new();
            for pair in body_str.split('&') {
                let mut parts = pair.split('=');
                let key = parts.next().unwrap();
                let value = parts.next().unwrap();
                form.insert(key.to_string(), value.to_string());
            }
            Ok(form)
        } else {
            Err(RequestError::InvalidContentType)
        }
    }
}
