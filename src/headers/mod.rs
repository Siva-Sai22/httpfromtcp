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

        let mut rest = request_string;
        loop {
            let index = match rest.find("\r\n") {
                Some(i) => i,
                None => return Ok((false, (request_string.len() - rest.len()))),
            };

            if index == 0 {
                return Ok((true, (request_string.len() - rest.len())));
            }

            let header_line = rest[..index].trim().to_string();

            let seperator_index = match header_line.find(':') {
                Some(i) => i,
                None => return Err(error_malformed_header),
            };

            // TODO: Its better to use bytes only not String
            if header_line.as_bytes()[seperator_index - 1] == b' ' {
                return Err(error_malformed_header);
            }

            self.0.insert(
                header_line[..seperator_index].to_string(),
                header_line[(seperator_index + 1)..].trim().to_string(),
            );

            rest = &rest[index + 2..];
        }
    }

    fn new() -> Headers {
        Headers(HashMap::new())
    }
}

#[cfg(test)]
mod test;
