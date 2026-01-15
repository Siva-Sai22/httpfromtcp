use std::io::Error;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::headers::Headers;

#[repr(u16)]
pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    InternalServerError = 500,
}

pub async fn write_status_line<W>(stream: &mut W, status_code: StatusCode) -> Result<(), Error>
where
    W: AsyncWrite + Unpin,
{
    let reason = match status_code {
        StatusCode::Ok => "Ok",
        StatusCode::BadRequest => "Bad Request",
        StatusCode::InternalServerError => "Internal Server Error",
    };

    stream
        .write_all(format!("HTTP/1.1 {} {}\r\n", status_code as u16, reason).as_bytes())
        .await?;

    Ok(())
}

pub fn get_default_headers(content_len: u16) -> Headers {
    let mut headers = Headers::new();

    headers.set("Content-length", &content_len.to_string());
    headers.set("Connection", "close");
    headers.set("Content-Type", "text/plain");

    headers
}

pub async fn write_headers<W>(stream: &mut W, headers: Headers) -> Result<(), Error>
where
    W: AsyncWrite + Unpin,
{
    for (key, value) in headers.headers {
        stream
            .write_all(format!("{}: {}\r\n", key, value).as_bytes())
            .await?;
    }

    stream.write_all(b"\r\n").await?;

    Ok(())
}
