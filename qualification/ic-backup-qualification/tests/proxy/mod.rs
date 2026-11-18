//! Loopback fixture proxy: forward once, retain original wire bytes, optionally lose one reply.
//! It is not a Backup transport or a production HTTP implementation.
use ic_platform::ingress::{CallResponse, Content, decode_call_response, decode_envelope};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

pub(super) struct Proxy {
    pub endpoint: String,
    pub calls: Arc<AtomicUsize>,
    pub lost: Arc<Mutex<Option<Vec<u8>>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Proxy {
    pub fn start(
        backend: &str,
        directory: PathBuf,
        lose: Option<(&'static str, usize)>,
        deny_write: Option<(&'static str, PathBuf)>,
    ) -> Self {
        let address = backend
            .strip_prefix("http://")
            .unwrap()
            .trim_end_matches('/')
            .to_owned();
        assert!(address.starts_with("127.0.0.1:"));
        fs::create_dir(&directory).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let calls = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let lost = Arc::new(Mutex::new(None));
        let worker = {
            let calls = Arc::clone(&calls);
            let stop = Arc::clone(&stop);
            let lost = Arc::clone(&lost);
            thread::spawn(move || {
                let mut discarded = false;
                let mut matched = 0;
                while !stop.load(Ordering::SeqCst) {
                    let mut client = match listener.accept() {
                        Ok((socket, _)) => socket,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                            continue;
                        }
                        Err(e) => panic!("proxy accept: {e}"),
                    };
                    // Accepted sockets can inherit nonblocking mode on macOS.
                    client.set_nonblocking(false).unwrap();
                    client
                        .set_read_timeout(Some(Duration::from_secs(60)))
                        .unwrap();
                    client
                        .set_write_timeout(Some(Duration::from_secs(60)))
                        .unwrap();
                    let request = read_request(&mut client);
                    let n = calls.fetch_add(1, Ordering::SeqCst);
                    fs::write(directory.join(format!("request-{n}.http")), &request).unwrap();
                    let split = request.windows(4).position(|b| b == b"\r\n\r\n").unwrap();
                    let head = std::str::from_utf8(&request[..split]).unwrap();
                    let mut forwarded = head
                        .lines()
                        .filter(|line| {
                            !line.to_ascii_lowercase().starts_with("host:")
                                && !line.to_ascii_lowercase().starts_with("connection:")
                        })
                        .collect::<Vec<_>>()
                        .join("\r\n")
                        .into_bytes();
                    forwarded.extend_from_slice(
                        format!("\r\nHost: {address}\r\nConnection: close\r\n\r\n").as_bytes(),
                    );
                    forwarded.extend_from_slice(&request[split + 4..]);
                    let mut upstream = TcpStream::connect(&address).unwrap();
                    upstream
                        .set_read_timeout(Some(Duration::from_secs(60)))
                        .unwrap();
                    upstream.write_all(&forwarded).unwrap();
                    let mut response = Vec::new();
                    upstream
                        .take(3 * 1024 * 1024 + 1)
                        .read_to_end(&mut response)
                        .unwrap();
                    assert!(response.len() <= 3 * 1024 * 1024);
                    fs::write(directory.join(format!("response-{n}.http")), &response).unwrap();
                    let method = decode_envelope(&request[split + 4..], 2 * 1024 * 1024)
                        .ok()
                        .and_then(|packet| match packet.value.content {
                            Content::Call { method_name, .. } => Some(method_name),
                            _ => None,
                        });
                    if let Some((selected, path)) = &deny_write
                        && method.as_deref() == Some(selected)
                    {
                        use std::os::unix::fs::PermissionsExt;
                        fs::set_permissions(path, fs::Permissions::from_mode(0o500)).unwrap();
                    }
                    let discard = if let Some((selected, ordinal)) = lose {
                        if method.as_deref() == Some(selected) {
                            let discard = matched == ordinal;
                            matched += 1;
                            discard
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !discarded && discard {
                        let body = body(&response);
                        let CallResponse::Replied { certificate } =
                            decode_call_response(&body, 2 * 1024 * 1024).unwrap().value
                        else {
                            panic!("lost response must be certified terminal")
                        };
                        let mut file = fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(directory.join("surviving-certificate.bin"))
                            .unwrap();
                        file.write_all(&certificate).unwrap();
                        file.sync_all().unwrap();
                        *lost.lock().unwrap() = Some(certificate);
                        discarded = true;
                    } else {
                        client.write_all(&response).unwrap();
                    }
                }
            })
        };
        Self {
            endpoint,
            calls,
            lost,
            stop,
            worker: Some(worker),
        }
    }

    pub fn finish(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.worker.take().unwrap().join().unwrap();
    }
}
impl Drop for Proxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut scratch = [0; 4096];
    loop {
        let n = stream.read(&mut scratch).unwrap();
        assert_ne!(n, 0);
        bytes.extend_from_slice(&scratch[..n]);
        assert!(bytes.len() <= 2 * 1024 * 1024 + 16384);
        if let Some(split) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
            let length: usize = std::str::from_utf8(&bytes[..split])
                .unwrap()
                .lines()
                .find_map(|line| {
                    line.split_once(':')
                        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                        .map(|(_, value)| value.trim().parse().unwrap())
                })
                .unwrap();
            if bytes.len() == split + 4 + length {
                return bytes;
            }
        }
    }
}

fn body(response: &[u8]) -> Vec<u8> {
    let split = response.windows(4).position(|b| b == b"\r\n\r\n").unwrap();
    let head = std::str::from_utf8(&response[..split]).unwrap();
    assert!(head.starts_with("HTTP/1.1 200"));
    let bytes = &response[split + 4..];
    if !head
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked")
    {
        return bytes.to_vec();
    }
    let mut remaining = bytes;
    let mut out = Vec::new();
    loop {
        let split = remaining.windows(2).position(|b| b == b"\r\n").unwrap();
        let size = usize::from_str_radix(
            std::str::from_utf8(&remaining[..split])
                .unwrap()
                .split(';')
                .next()
                .unwrap(),
            16,
        )
        .unwrap();
        if size == 0 {
            return out;
        }
        remaining = &remaining[split + 2..];
        out.extend_from_slice(&remaining[..size]);
        remaining = &remaining[size + 2..];
    }
}
