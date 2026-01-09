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
