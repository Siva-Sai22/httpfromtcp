use super::*;

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::ReadBuf;

#[derive(Debug)]
struct ChunkReader {
    data: Vec<u8>,
    num_bytes_per_read: usize,
    pos: usize,
}

impl AsyncRead for ChunkReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        if self.pos >= self.data.len() {
            return Poll::Ready(Ok(()));
        }
        let end = (self.pos + self.num_bytes_per_read).min(self.data.len());
        let to_copy = &self.data[self.pos..end];
        buf.put_slice(to_copy);
        self.pos += to_copy.len();
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn good_get_request_line() {
    let req_bytes = b"GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 3,
        pos: 0,
    };

    let result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(RequestMethod::Get, result.request_line.method);
    assert_eq!("/", result.request_line.request_target);
    assert_eq!("1.1", result.request_line.http_version);
}

#[tokio::test]
async fn good_get_request_line_with_path() {
    let req_bytes = b"GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 1,
        pos: 0,
    };

    let result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(RequestMethod::Get, result.request_line.method);
    assert_eq!("/coffee", result.request_line.request_target);
    assert_eq!("1.1", result.request_line.http_version);
}

#[tokio::test]
async fn invalid_number_of_parts_in_request_line() {
    let req_bytes = b"/coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 3,
        pos: 0,
    };

    let result = request_from_reader(reader).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn post_request_line() {
    let req_bytes = b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 3,
        pos: 0,
    };

    let result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(RequestMethod::Post, result.request_line.method);
    assert_eq!("/submit", result.request_line.request_target);
    assert_eq!("1.1", result.request_line.http_version);
}

#[tokio::test]
async fn invalid_method_request_line() {
    let req_bytes = b"FETCH /data HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 3,
        pos: 0,
    };

    let result = request_from_reader(reader).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn invalid_http_version_request_line() {
    let req_bytes = b"GET /data HTTP/2.0\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 3,
        pos: 0,
    };

    let result = request_from_reader(reader).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn valid_request_with_headers() {
    let req_bytes = b"GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 4,
        pos: 0,
    };

    let mut result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(RequestMethod::Get, result.request_line.method);
    assert_eq!("/", result.request_line.request_target);
    assert_eq!("1.1", result.request_line.http_version);
    assert_eq!(
        result.headers.get("host"),
        Some(&"localhost:42069".to_string())
    );
    assert_eq!(
        result.headers.get("user-agent"),
        Some(&"curl/7.81.0".to_string())
    );
    assert_eq!(result.headers.get("accept"), Some(&"*/*".to_string()));
}

#[tokio::test]
async fn malformed_header() {
    let req_bytes =
        b"GET / HTTP/1.1\r\nHost localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 4,
        pos: 0,
    };

    let result = request_from_reader(reader).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn empty_headers() {
    let req_bytes = b"GET / HTTP/1.1\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 5,
        pos: 0,
    };

    let result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(RequestMethod::Get, result.request_line.method);
    assert_eq!("/", result.request_line.request_target);
    assert_eq!("1.1", result.request_line.http_version);
}

#[tokio::test]
async fn standard_body() {
    let req_bytes = b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 13\r\n\r\nHello, world!";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 3,
        pos: 0,
    };

    let mut result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(RequestMethod::Post, result.request_line.method);
    assert_eq!("/submit", result.request_line.request_target);
    assert_eq!("1.1", result.request_line.http_version);
    assert_eq!(
        result.headers.get("content-length"),
        Some(&"13".to_string())
    );
    assert_eq!(result.body, b"Hello, world!");
}

#[tokio::test]
async fn more_content_than_content_length() {
    let req_bytes =
        b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 5\r\n\r\nHello, world!";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 3,
        pos: 0,
    };

    let result = request_from_reader(reader).await.unwrap();

    assert_eq!(result.body, b"Hello");
}

#[tokio::test]
async fn invalid_content_length() {
    let req_bytes = b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: abc\r\n\r\nHello, world!";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 3,
        pos: 0,
    };

    let result = request_from_reader(reader).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn empty_body_with_content_length() {
    let req_bytes = b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 0\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 4,
        pos: 0,
    };

    let mut result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(RequestMethod::Post, result.request_line.method);
    assert_eq!("/submit", result.request_line.request_target);
    assert_eq!("1.1", result.request_line.http_version);
    assert_eq!(result.headers.get("content-length"), Some(&"0".to_string()));
    assert_eq!(result.body, b"");
}

#[tokio::test]
async fn empty_body_without_content_length() {
    let req_bytes = b"GET / HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 4,
        pos: 0,
    };

    let result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(RequestMethod::Get, result.request_line.method);
    assert_eq!("/", result.request_line.request_target);
    assert_eq!("1.1", result.request_line.http_version);
    assert_eq!(result.body, b"");
}

#[tokio::test]
async fn body_without_content_length() {
    let req_bytes = b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\n\r\nHello, world!";
    let reader = ChunkReader {
        data: req_bytes.to_vec(),
        num_bytes_per_read: 5,
        pos: 0,
    };

    let result = request_from_reader(reader)
        .await
        .expect("Failed to parse request");

    assert_eq!(result.body, b"");
}
