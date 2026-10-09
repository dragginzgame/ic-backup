use super::*;
use std::{
    io::{Cursor, Write},
    net::TcpListener,
    thread,
};

fn request(body: &[u8]) -> Vec<u8> {
    let mut bytes =
        format!("POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

#[test]
fn interrupted_read_retains_exact_request_and_does_not_reset_deadline() {
    let expected = request(b"abc");
    let mut source = Cursor::new(expected.clone());
    let mut interrupted = false;
    let capture = read_request(
        |bytes, _| {
            if !interrupted {
                interrupted = true;
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            source.read(bytes)
        },
        Duration::from_secs(1),
    );
    assert!(capture.failure.is_none());
    assert_eq!(capture.bytes, expected);
    let exhausted = read_request(
        |_, _| Err(io::Error::from(io::ErrorKind::Interrupted)),
        Duration::from_millis(2),
    );
    assert!(matches!(exhausted.failure, Some(ReadFailure::Deadline)));
    assert_eq!(exhausted.bytes, [] as [u8; 0]);
}

#[test]
fn incomplete_eof_and_timeout_retain_original_bytes_and_read_error() {
    let partial = b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\nab";
    let mut source = Cursor::new(partial);
    let eof = read_request(|bytes, _| source.read(bytes), Duration::from_secs(1));
    assert!(matches!(eof.failure, Some(ReadFailure::Eof)));
    assert_eq!(eof.bytes, partial);
    for kind in [io::ErrorKind::WouldBlock, io::ErrorKind::TimedOut] {
        let mut source = Cursor::new(partial);
        let capture = read_request(
            |bytes, _| {
                let count = source.read(bytes)?;
                if count == 0 {
                    Err(io::Error::new(kind, "controlled original read error"))
                } else {
                    Ok(count)
                }
            },
            Duration::from_secs(1),
        );
        assert_eq!(capture.bytes, partial);
        let Some(ReadFailure::Io(error)) = &capture.failure else {
            panic!("original IO failure");
        };
        assert_eq!(error.kind(), kind);
        let root = crate::support::root("partial-http");
        capture.retain(&root, 0).unwrap();
        assert_eq!(fs::read(root.join("http-request-0.bin")).unwrap(), partial);
        let record: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("http-request-0.json")).unwrap()).unwrap();
        assert_eq!(record["complete"], false);
        assert_eq!(record["failure"]["kind"], format!("{kind:?}"));
        assert_eq!(
            record["failure"]["message"],
            "controlled original read error"
        );
    }
}

#[test]
fn malformed_framing_and_oversized_declarations_refuse_before_body_reads() {
    for header in [
        b"POST / HTTP/1.1\r\n\r\n".to_vec(),
        b"POST / HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\n".to_vec(),
        b"POST / HTTP/1.1\r\nContent-Length: +1\r\n\r\n".to_vec(),
        b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\nContent-Length: 0\r\n\r\n".to_vec(),
        format!(
            "POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_SIGNED_UPDATE_BYTES + 1
        )
        .into_bytes(),
    ] {
        let mut source = Cursor::new(&header);
        let mut calls = 0;
        let capture = read_request(
            |bytes, _| {
                calls += 1;
                source.read(bytes)
            },
            Duration::from_secs(1),
        );
        assert!(matches!(
            capture.failure,
            Some(ReadFailure::ContentLength | ReadFailure::BodyBound)
        ));
        assert_eq!(calls, 1);
        assert_eq!(capture.bytes, header);
    }
    let mut source = Cursor::new(vec![b'x'; MAX_HEADER_BYTES + 4096]);
    let capture = read_request(|bytes, _| source.read(bytes), Duration::from_secs(1));
    assert!(matches!(capture.failure, Some(ReadFailure::HeaderBound)));
    assert!(capture.bytes.len() <= MAX_REQUEST_BYTES);
    let mut trailing = request(b"abc");
    trailing.push(b'x');
    let mut source = Cursor::new(&trailing);
    let capture = read_request(|bytes, _| source.read(bytes), Duration::from_secs(1));
    assert!(matches!(capture.failure, Some(ReadFailure::TrailingBytes)));
    assert_eq!(capture.bytes, trailing);
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
    stream.set_nonblocking(true).unwrap(); // Reproduce inherited flags on every host.
    let capture = capture(&mut stream, Duration::from_secs(1));
    client.join().unwrap();
    assert!(capture.failure.is_none(), "{:?}", capture.failure);
    assert_eq!(capture.bytes, expected);
}

#[test]
fn a_live_incomplete_body_timeout_retains_the_original_os_error() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    let partial = b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\nab";
    client.write_all(partial).unwrap();
    let capture = capture(&mut stream, Duration::from_millis(30));
    assert_eq!(capture.bytes, partial);
    let Some(ReadFailure::Io(error)) = &capture.failure else {
        panic!("actual socket read error: {:?}", capture.failure);
    };
    assert!(matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    ));
    assert!(error.raw_os_error().is_some());
}
