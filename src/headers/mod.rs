use std::{collections::HashMap, io::Error};

struct Headers(HashMap<String, String>);

impl Headers {
    fn parse(&mut self, buffer: &[u8]) -> Result<(bool, usize), Error> {
        let error_malformed_header =
            Error::new(std::io::ErrorKind::InvalidData, "Malformed Header");

        let request_string = match std::str::from_utf8(buffer) {
            Ok(s) => s,
            Err(_) => return Err(error_malformed_header),
        };

        let index = match request_string.find("\r\n") {
            Some(i) => i,
            None => return Ok((false, 0)),
        };

        if index == 0 {
            return Ok((true, "\r\n".len()));
        }

        let header_line = request_string[..index].trim().to_string();
        // TODO: Its better to use bytes only not String
        if header_line.as_bytes()[4] != b':' {
            return Err(error_malformed_header);
        }

        self.0.insert(
            header_line[..4].to_string(),
            header_line[5..].trim().to_string(),
        );

        return Ok((true, index + 2));
    }

    fn new() -> Headers {
        Headers(HashMap::new())
    }
}

#[cfg(test)]
mod test;
