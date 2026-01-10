use std::{collections::HashMap, io::Error};

pub struct Headers(pub HashMap<String, String>);

impl Headers {
    pub fn parse(&mut self, buffer: &[u8]) -> Result<(bool, usize), Error> {
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

            let separator_index = match header_line.find(':') {
                Some(i) => i,
                None => return Err(error_malformed_header),
            };

            let field_name = &header_line[..separator_index];
            let field_value = header_line[(separator_index + 1)..].trim();

            // Validate field-name characters
            if !field_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c))
            {
                return Err(error_malformed_header);
            }

            // TODO: Use bytes everywhere rather than String
            if separator_index > 0 && header_line.as_bytes()[separator_index - 1] == b' ' {
                return Err(error_malformed_header);
            }

            if let Some(existing_value) = self.0.get(field_name.to_lowercase().as_str()) {
                let new_value = format!("{}, {}", existing_value, field_value);
                self.0.insert(field_name.to_lowercase(), new_value);
            } else {
                self.0
                    .insert(field_name.to_lowercase(), field_value.to_string());
            }

            rest = &rest[index + 2..];
        }
    }

    pub fn new() -> Headers {
        Headers(HashMap::new())
    }
}

#[cfg(test)]
mod test;
