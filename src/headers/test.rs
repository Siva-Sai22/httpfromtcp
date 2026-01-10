use super::*;

#[test]
fn valid_single_header() {
    let mut headers = Headers::new();
    let data = b"Host: localhost:42069\r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(n, 25);
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

    assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(n, 37);
    assert!(done);
}

#[test]
fn valid_two_headers() {
    let mut headers = Headers::new();
    let data = b"Host: localhost:42069\r\nUser-Agent: TestAgent\r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(headers.get("User-Agent"), Some(&"TestAgent".to_string()));
    assert_eq!(n, 48);
    assert!(done);
}

#[test]
fn valid_two_header_with_existing_headers() {
    let mut headers = Headers::new();
    headers.set("existing", "Header");
    let data = b"Host: localhost:42069\r\nUser-Agent: TestAgent\r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert_eq!(headers.get("existing"), Some(&"Header".to_string()));
    assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(headers.get("User-Agent"), Some(&"TestAgent".to_string()));
    assert_eq!(n, 48);
    assert!(done);
}

#[test]
fn capital_header_names() {
    let mut headers = Headers::new();
    let data = b"hOsT: localhost:42069\r\nuSeR-aGeNt: TestAgent\r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(headers.get("User-Agent"), Some(&"TestAgent".to_string()));
    assert_eq!(n, 48);
    assert!(done);
}

#[test]
fn multiple_values_for_single_header() {
    let mut headers = Headers::new();
    let data = b"Cookie: value1\r\nCookie: value2\r\n\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert_eq!(headers.get("Cookie"), Some(&"value1, value2".to_string()));
    assert_eq!(n, 34);
    assert!(done);
}

#[test]
fn missing_ending_crlf() {
    let mut headers = Headers::new();
    let data = b"Host: localhost:42069\r\nUser-Agent: TestAgent\r\n";

    let result = headers.parse(data);
    let (done, n) = result.unwrap();

    assert_eq!(headers.get("Host"), Some(&"localhost:42069".to_string()));
    assert_eq!(headers.get("User-Agent"), Some(&"TestAgent".to_string()));
    assert_eq!(n, 46);
    assert!(!done);
}
