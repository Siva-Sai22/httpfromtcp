use std::{cmp::min, io::Error};

use crate::headers::Headers;

use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, PartialEq)]
pub enum RequestMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl RequestMethod {
    fn from_str(method: &str) -> Option<RequestMethod> {
        match method {
            "GET" => Some(RequestMethod::Get),
            "POST" => Some(RequestMethod::Post),
            "PUT" => Some(RequestMethod::Put),
            "DELETE" => Some(RequestMethod::Delete),
            _ => None,
        }
    }
}

pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: RequestMethod,
}

#[derive(PartialEq)]
enum ParserState {
    StateRequestLine,
    StateHeaders,
    StateBody,
    Done,
}

pub struct Request {
    pub request_line: RequestLine,
    pub headers: Headers,
    pub body: Vec<u8>,
    state: ParserState,
}

fn new_request() -> Request {
    Request {
        request_line: RequestLine {
            http_version: String::new(),
            request_target: String::new(),
            method: RequestMethod::Get,
        },
        headers: Headers::new(),
        body: Vec::new(),
        state: ParserState::StateRequestLine,
    }
}

// e.g. : GET /coffee HTTP/1.1
fn parse_request_line(request: &[u8]) -> Result<(Option<RequestLine>, usize), Error> {
    let error_malformed_request_line =
        Error::new(std::io::ErrorKind::InvalidData, "Malformed Request Line");
    let error_unsupported_http_version =
        Error::new(std::io::ErrorKind::InvalidData, "Unsupported HTTP Version");
    let error_invalid_request_method =
        Error::new(std::io::ErrorKind::InvalidData, "Invalid Request Method");

    let request_string = match std::str::from_utf8(request) {
        Ok(s) => s,
        Err(_) => return Err(error_malformed_request_line),
    };
    let index = match request_string.find("\r\n") {
        Some(i) => i,
        None => return Ok((None, 0)),
    };

    let request_line = &request_string[..index];
    let read = index + "\r\n".len();

    let parts = request_line.split_whitespace().collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(error_malformed_request_line);
    }

    let method = match RequestMethod::from_str(parts[0]) {
        Some(m) => m,
        None => return Err(error_invalid_request_method),
    };

    let http_parts = parts[2].split("/").collect::<Vec<&str>>();
    if http_parts[0] != "HTTP" || http_parts[1] != "1.1" {
        return Err(error_unsupported_http_version);
    }

    Ok((
        Some(RequestLine {
            http_version: http_parts[1].to_string(),
            request_target: parts[1].to_string(),
            method,
        }),
        read,
    ))
}

impl Request {
    fn parse(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        match self.state {
            ParserState::StateRequestLine => match parse_request_line(buffer) {
                Ok((request_line, bytes_parsed)) => {
                    if bytes_parsed == 0 {
                        return Ok(0);
                    }

                    // Only returns none when bytes_parsed == 0. Covered above
                    self.request_line = request_line.unwrap();
                    self.state = ParserState::StateHeaders;

                    Ok(bytes_parsed)
                }
                Err(e) => Err(e),
            },
            ParserState::StateHeaders => match self.headers.parse(buffer) {
                Ok((done, bytes_parsed)) => {
                    if done {
                        self.state = ParserState::StateBody;
                    }
                    Ok(bytes_parsed)
                }
                Err(e) => Err(e),
            },
            ParserState::StateBody => {
                let content_length = if let Some(c) = self.headers.get("content-length") {
                    match c.parse::<usize>() {
                        Ok(t) => t,
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Malformed Content-Length Header",
                            ));
                        }
                    }
                } else {
                    self.state = ParserState::Done;
                    return Ok(0);
                };

                let remaining = min(buffer.len(), content_length - self.body.len());
                self.body.extend_from_slice(&buffer[..remaining]);

                if self.body.len() == content_length {
                    self.state = ParserState::Done;
                }

                Ok(buffer.len())
            }
            ParserState::Done => Ok(0),
        }
    }
}

pub async fn request_from_reader<R>(mut stream: R) -> Result<Request, Error>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = [0u8; 1024];
    let mut request = new_request();
    let mut buf_len = 0;

    while request.state != ParserState::Done {
        let bytes_read = match stream.read(&mut buffer[buf_len..]).await {
            Ok(n) => n,
            // TODO: Should resolve the errors
            Err(e) => return Err(e),
        };
        buf_len += bytes_read;

        let read_bytes = match request.parse(&buffer[..buf_len]) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        buffer.copy_within(read_bytes..buf_len, 0);
        buf_len -= read_bytes;
    }

    Ok(request)
}

#[cfg(test)]
mod test;
