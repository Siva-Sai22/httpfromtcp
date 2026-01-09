use super::*;

#[test]
fn valid_single_header() {
    let mut headers = Headers::new();
    let data = b"Host: localhost:42069\r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert!(!headers.0.is_empty());
    assert_eq!(headers.0.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(n, 23);
    assert!(done);
}

#[test]
fn invalid_spacing_header() {
    let mut headers = Headers::new();
    let data = b"       Host : localhost:42069       \r\n\r\n";

    let result = headers.parse(data);

    assert!(result.is_err());
}

#[test]
fn valid_single_header_with_spaces() {
    let mut headers = Headers::new();
    let data = b"     Host:    localhost:42069    \r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert!(!headers.0.is_empty());
    assert_eq!(headers.0.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(n, 35);
    assert!(done);
}

#[test]
fn valid_two_headers() {
    let mut headers = Headers::new();
    let data = b"Host: localhost:42069\r\nUser-Agent: TestAgent\r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert!(!headers.0.is_empty());
    assert_eq!(headers.0.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(headers.0.get("User-Agent"), Some(&"TestAgent".to_string()));
    assert_eq!(n, 46);
    assert!(done);
}

#[test]
fn valid_two_header_with_existing_headers() {
    let mut headers = Headers::new();
    headers
        .0
        .insert("Existing".to_string(), "Header".to_string());
    let data = b"Host: localhost:42069\r\nUser-Agent: TestAgent\r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert!(!headers.0.is_empty());
    assert_eq!(headers.0.get("Existing"), Some(&"Header".to_string()));
    assert_eq!(headers.0.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(headers.0.get("User-Agent"), Some(&"TestAgent".to_string()));
    assert_eq!(n, 46);
    assert!(done);
}
