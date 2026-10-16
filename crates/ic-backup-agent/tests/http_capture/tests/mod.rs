use super::*;
use std::{io::Write, net::TcpListener, thread};

fn request(body: &[u8]) -> Vec<u8> {
    let mut bytes =
        format!("POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

#[test]
fn a_nonblocking_accepted_stream_is_normalized_before_fragmented_body_capture() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let expected = request(b"abc");
    let bytes = expected.clone();
    let address = listener.local_addr().unwrap();
    let client = thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(&bytes[..bytes.len() - 2]).unwrap();
        thread::sleep(Duration::from_millis(20));
        stream.write_all(&bytes[bytes.len() - 2..]).unwrap();
    });
    let (mut stream, _) = listener.accept().unwrap();
    stream.set_nonblocking(true).unwrap();
    let capture = capture(&mut stream, Duration::from_secs(1));
    client.join().unwrap();
    assert!(capture.failure.is_none(), "{:?}", capture.failure);
    assert_eq!(capture.bytes, expected);
}

#[test]
fn a_live_incomplete_body_retains_exact_socket_and_json_evidence() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    let partial = b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\nab";
    client.write_all(partial).unwrap();
    let capture = capture(&mut stream, Duration::from_millis(100));
    assert_eq!(capture.bytes, partial);
    let Some(HttpCaptureFailure::Io(error)) = &capture.failure else {
        panic!("actual socket read error: {:?}", capture.failure);
    };
    assert!(matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    ));
    assert!(error.raw_os_error().is_some());
    let root = crate::support::root("partial-http");
    capture.retain(&root, 0).unwrap();
    assert_eq!(fs::read(root.join("http-request-0.bin")).unwrap(), partial);
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("http-request-0.json")).unwrap()).unwrap();
    assert_eq!(record["complete"], false);
    assert_eq!(record["bytes"], partial.len());
    assert_eq!(record["failure"]["kind"], format!("{:?}", error.kind()));
    assert_eq!(
        record["failure"]["raw_os_error"],
        serde_json::json!(error.raw_os_error())
    );
    assert_eq!(record["failure"]["message"], error.to_string());
}

#[test]
fn a_live_missing_content_length_is_refused_with_original_bytes_retained() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    let header = b"POST / HTTP/1.1\r\n\r\n";
    client.write_all(header).unwrap();
    let capture = capture(&mut stream, Duration::from_secs(1));
    assert!(matches!(
        capture.failure,
        Some(HttpCaptureFailure::ContentLength)
    ));
    assert_eq!(capture.bytes, header);
}
