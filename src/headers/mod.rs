use std::{collections::HashMap, io::Error};

fn is_token(field_name: &str) -> bool {
    if field_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c))
    {
        return true;
    }

    false
}

fn parse_header(header_line: &str) -> Result<(String, String), Error> {
    let error_malformed_header = Error::new(std::io::ErrorKind::InvalidData, "Malformed Header");

    let separator_index = match header_line.find(':') {
        Some(i) => i,
        None => return Err(error_malformed_header),
    };

    let field_name = &header_line[..separator_index];
    let field_value = header_line[(separator_index + 1)..].trim();

    if !is_token(field_name) {
        return Err(error_malformed_header);
    }

    // TODO: Use bytes everywhere rather than String
    if separator_index > 0 && header_line.as_bytes()[separator_index - 1] == b' ' {
        return Err(error_malformed_header);
    }

    Ok((field_name.to_string(), field_value.to_string()))
}

pub struct Headers {
    pub headers: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            headers: HashMap::new(),
        }
    }
    pub fn get(&mut self, key: &str) -> Option<&String> {
        self.headers.get(&key.to_lowercase())
    }

    pub fn for_each(&mut self, f: fn(&String, &String)) {
        for (key, value) in self.headers.iter() {
            f(key, value);
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if let Some(existing_value) = self.get(key.to_lowercase().as_str()) {
            let new_value = format!("{}, {}", existing_value, value);
            self.headers.insert(key.to_lowercase(), new_value);
        } else {
            self.headers.insert(key.to_lowercase(), value.to_string());
        }
    }

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
                return Ok((true, (request_string.len() - rest.len() + 2)));
            }

            let header_line = rest[..index].trim().to_string();

            let (field_name, field_value) = match parse_header(&header_line) {
                Ok((s, t)) => (s, t),
                Err(e) => return Err(e),
            };

            self.set(&field_name, &field_value);

            rest = &rest[index + 2..];
        }
    }
}

#[cfg(test)]
mod test;
