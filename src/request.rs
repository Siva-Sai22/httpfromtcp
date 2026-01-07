use std::io::Error;

use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, PartialEq)]
enum RequestMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl RequestMethod {
    fn from_str(method: &str) -> Option<RequestMethod> {
        match method {
            "GET" => Some(RequestMethod::GET),
            "POST" => Some(RequestMethod::POST),
            "PUT" => Some(RequestMethod::PUT),
            "DELETE" => Some(RequestMethod::DELETE),
            _ => None,
        }
    }
}

struct RequestLine {
    http_version: String,
    request_target: String,
    method: RequestMethod,
}

struct Request {
    request_line: RequestLine,
}

// e.g. : GET /coffee HTTP/1.1
fn parse_request_line(request: &str) -> Result<RequestLine, Error> {
    let invalid_request_err = Error::new(std::io::ErrorKind::InvalidData, "Invalid Request");

    let request_line = if let Some(index) = request.find("\r\n") {
        &request[..index]
    } else {
        return Err(invalid_request_err);
    };

    let parts = request_line.split_whitespace().collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(invalid_request_err);
    }

    let method = match RequestMethod::from_str(parts[0]) {
        Some(m) => m,
        None => return Err(invalid_request_err),
    };

    let http_version = if let Some(index) = parts[2].find('/') {
        &parts[2][index + 1..]
    } else {
        return Err(invalid_request_err);
    };
    if !http_version.eq("1.1") {
        return Err(invalid_request_err);
    }

    return Ok(RequestLine {
        http_version: http_version.to_string(),
        request_target: parts[1].to_string(),
        method,
    });
}

async fn request_from_reader<R>(mut stream: R) -> Result<Request, Error>
where
    R: AsyncRead + Unpin,
{
    let mut request = String::new();

    if let Err(e) = stream.read_to_string(&mut request).await {
        return Err(e);
    }

    let request_line: RequestLine;
    match parse_request_line(&request) {
        Ok(s) => request_line = s,
        Err(e) => return Err(e),
    }

    return Ok(Request { request_line });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, duplex};

    #[tokio::test]
    async fn test_good_get_request_line() {
        let req_bytes = b"GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let (mut writer, reader) = duplex(1024);

        tokio::spawn(async move {
            writer.write_all(req_bytes).await.unwrap();
            let _ = writer.shutdown().await;
        });

        let result = request_from_reader(reader)
            .await
            .expect("Failed to parse request");
        assert_eq!(RequestMethod::GET, result.request_line.method);
        assert_eq!("/", result.request_line.request_target);
        assert_eq!("1.1", result.request_line.http_version);
    }

    #[tokio::test]
    async fn test_good_get_request_line_with_path() {
        let req_bytes = b"GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let (mut writer, reader) = duplex(1024);

        tokio::spawn(async move {
            writer.write_all(req_bytes).await.unwrap();
            let _ = writer.shutdown().await;
        });

        let result = request_from_reader(reader)
            .await
            .expect("Failed to parse request");
        assert_eq!(RequestMethod::GET, result.request_line.method);
        assert_eq!("/coffee", result.request_line.request_target);
        assert_eq!("1.1", result.request_line.http_version);
    }

    #[tokio::test]
    async fn test_invalid_number_of_parts_in_request_line() {
        let req_bytes = b"/coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let (mut writer, reader) = duplex(1024);

        tokio::spawn(async move {
            writer.write_all(req_bytes).await.unwrap();
            let _ = writer.shutdown().await;
        });

        let result = request_from_reader(reader).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_post_request_line() {
        let req_bytes = b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let (mut writer, reader) = duplex(1024);

        tokio::spawn(async move {
            writer.write_all(req_bytes).await.unwrap();
            let _ = writer.shutdown().await;
        });

        let result = request_from_reader(reader)
            .await
            .expect("Failed to parse request");
        assert_eq!(RequestMethod::POST, result.request_line.method);
        assert_eq!("/submit", result.request_line.request_target);
        assert_eq!("1.1", result.request_line.http_version);
    }
    
    #[tokio::test]
    async fn test_invalid_method_request_line() {
        let req_bytes = b"FETCH /data HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let (mut writer, reader) = duplex(1024);

        tokio::spawn(async move {
            writer.write_all(req_bytes).await.unwrap();
            let _ = writer.shutdown().await;
        });

        let result = request_from_reader(reader).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_invalid_http_version_request_line() {
        let req_bytes = b"GET /data HTTP/2.0\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let (mut writer, reader) = duplex(1024);

        tokio::spawn(async move {
            writer.write_all(req_bytes).await.unwrap();
            let _ = writer.shutdown().await;
        });

        let result = request_from_reader(reader).await;
        assert!(result.is_err());
    }
}
