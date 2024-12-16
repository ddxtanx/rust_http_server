use std::collections::HashMap as Map;

use crate::parsing::HttpMethod;

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    query: Map<String, String>,
    headers: Map<String, String>,
    body: Option<Vec<u8>>,
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
}
