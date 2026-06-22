//! An HTTP/1.1 server that serves the dashboard over a pluggable byte transport.
//!
//! This is the host side of the local-first dashboard: it serves the static page, the
//! language-neutral `GET /state` snapshot, and a `GET /events` server-sent-event
//! stream for live updates, with no web framework and no async runtime. A thread per
//! connection keeps it simple and is ample for the handful of clients a node sees over
//! its own hotspot. The same code backs the [`Mock`](crate::Mock) in development and a
//! real node in the field, since both are just a [`StateSource`].
//!
//! ## Why HTTP/1.1, and the seam for more
//!
//! HTTP/1.1 is the baseline on purpose: it is exactly what a browser speaks to a
//! device over a plain `http://` hotspot, where there is no CA-trusted certificate and
//! so no HTTPS (and therefore no browser HTTP/2, which is only negotiated over TLS).
//! It also fits the smallest tiers, where a TLS plus HTTP/2 stack would not. The
//! request handling is generic over any [`Read`] + [`Write`] stream, and connections
//! arrive through a [`Transport`], so a capable tier can later supply a TLS transport
//! (which negotiates HTTP/2 for free in the browser) without touching the request
//! logic. [`TcpTransport`] is the plain-TCP baseline; a `rustls`-backed transport is
//! the intended Tier A addition.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::assets::Assets;
use crate::source::StateSource;

/// How connections reach the server: the seam that keeps the byte transport pluggable.
///
/// The baseline is [`TcpTransport`] (plain TCP, HTTP/1.1). A capable tier can
/// implement this over a TLS stream so the browser negotiates HTTPS, and with it
/// HTTP/2, while the request handling above stays unchanged.
pub trait Transport: Send + 'static {
    /// The connection type this transport yields, a bidirectional byte stream.
    type Conn: Read + Write + Send + 'static;

    /// Blocks until the next client connects.
    ///
    /// # Returns
    ///
    /// The accepted connection.
    ///
    /// # Errors
    ///
    /// Returns the [`std::io::Error`] from the underlying accept call.
    fn accept(&self) -> std::io::Result<Self::Conn>;

    /// A human-readable address for the startup log line, such as a URL.
    ///
    /// # Returns
    ///
    /// The address to print when the server starts serving.
    fn describe(&self) -> String;
}

/// The plain-TCP, HTTP/1.1 transport: the baseline that works on any tier.
pub struct TcpTransport {
    listener: TcpListener,
}

impl TcpTransport {
    /// Binds a listener on `addr`.
    ///
    /// # Arguments
    ///
    /// * `addr` - the address to listen on, such as `"0.0.0.0:80"`.
    ///
    /// # Returns
    ///
    /// A transport ready to accept connections.
    ///
    /// # Errors
    ///
    /// Returns the [`std::io::Error`] from binding if the address is unavailable.
    pub fn bind(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr)?,
        })
    }
}

impl Transport for TcpTransport {
    type Conn = TcpStream;

    fn accept(&self) -> std::io::Result<Self::Conn> {
        self.listener.accept().map(|(stream, _)| stream)
    }

    fn describe(&self) -> String {
        match self.listener.local_addr() {
            Ok(addr) => format!("http://{addr}"),
            Err(_) => "http://?".to_owned(),
        }
    }
}

/// The dashboard HTTP server, generic over whatever produces its state.
///
/// Build one with [`Server::new`], optionally set the live-update cadence with
/// [`Server::with_push_interval`], then block in [`Server::run`] (plain TCP) or
/// [`Server::run_on`] (a custom [`Transport`]).
///
/// # Examples
///
/// ```no_run
/// use pamoja_dashboard::{Assets, Mock, Scenario, Server};
///
/// let server = Server::new(Mock::new(Scenario::Normal), Assets::Embedded);
/// server.run("127.0.0.1:8080").expect("serve");
/// ```
pub struct Server<S> {
    source: Arc<Mutex<S>>,
    assets: Assets,
    push_interval: Duration,
}

impl<S: StateSource + Send + 'static> Server<S> {
    /// Creates a server that renders `source` with `assets`.
    ///
    /// # Arguments
    ///
    /// * `source` - the state source to serve, a real node or a [`Mock`](crate::Mock).
    /// * `assets` - where the page files come from, embedded or a directory.
    ///
    /// # Returns
    ///
    /// A server pushing live updates once a second by default.
    pub fn new(source: S, assets: Assets) -> Self {
        Self {
            source: Arc::new(Mutex::new(source)),
            assets,
            push_interval: Duration::from_secs(1),
        }
    }

    /// Sets how often the `GET /events` stream pushes a fresh snapshot.
    ///
    /// # Arguments
    ///
    /// * `interval` - the delay between pushes.
    ///
    /// # Returns
    ///
    /// The server, for chaining.
    pub fn with_push_interval(mut self, interval: Duration) -> Self {
        self.push_interval = interval;
        self
    }

    /// Binds `addr` over plain TCP and serves forever.
    ///
    /// # Arguments
    ///
    /// * `addr` - the address to listen on, such as `"0.0.0.0:80"`.
    ///
    /// # Returns
    ///
    /// Never returns on success; it serves until the process ends.
    ///
    /// # Errors
    ///
    /// Returns the [`std::io::Error`] from binding the listener.
    pub fn run(self, addr: impl ToSocketAddrs) -> std::io::Result<()> {
        let transport = TcpTransport::bind(addr)?;
        self.run_on(transport)
    }

    /// Serves forever over a supplied [`Transport`], one thread per connection.
    ///
    /// This is the seam for a non-default transport, such as a future TLS transport
    /// for a capable tier.
    ///
    /// # Arguments
    ///
    /// * `transport` - the source of connections.
    ///
    /// # Returns
    ///
    /// Never returns on success; it serves until the process ends.
    ///
    /// # Errors
    ///
    /// Returns a [`std::io::Error`] only if accepting fails unrecoverably; transient
    /// accept errors are skipped.
    pub fn run_on<T: Transport>(self, transport: T) -> std::io::Result<()> {
        println!("pamoja-dashboard: serving on {}", transport.describe());
        loop {
            let conn = match transport.accept() {
                Ok(conn) => conn,
                Err(_) => continue,
            };
            let source = Arc::clone(&self.source);
            let assets = self.assets.clone();
            let interval = self.push_interval;
            thread::spawn(move || {
                let _ = handle(conn, source, assets, interval);
            });
        }
    }
}

// One parsed request line: the method, the path, and the raw query string.
struct Request {
    method: String,
    path: String,
    query: String,
}

fn handle<S: StateSource, C: Read + Write>(
    mut conn: C,
    source: Arc<Mutex<S>>,
    assets: Assets,
    interval: Duration,
) -> std::io::Result<()> {
    let request = match read_request(&mut conn)? {
        Some(request) => request,
        None => return Ok(()),
    };

    // A `?scenario=` parameter is a dev affordance: it asks the source to switch view.
    if let Some(scenario) = query_value(&request.query, "scenario") {
        if let Ok(mut source) = source.lock() {
            source.select(&scenario);
        }
    }

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/state") => {
            let json = snapshot_json(&source);
            write_response(
                &mut conn,
                200,
                "OK",
                "application/json; charset=utf-8",
                json.as_bytes(),
            )
        }
        ("GET", "/events") => stream_events(&mut conn, &source, interval),
        ("GET", path) => match assets.get(path) {
            Some((content_type, bytes)) => {
                write_response(&mut conn, 200, "OK", content_type, &bytes)
            }
            None => write_response(&mut conn, 404, "Not Found", "text/plain", b"not found"),
        },
        ("POST", path) if path.starts_with("/command/") => write_response(
            &mut conn,
            501,
            "Not Implemented",
            "application/json; charset=utf-8",
            br#"{"error":"control.not_yet_authenticated"}"#,
        ),
        _ => write_response(
            &mut conn,
            405,
            "Method Not Allowed",
            "text/plain",
            b"method not allowed",
        ),
    }
}

// Reads and parses the request line and drains the headers and any body. Returns
// `None` on an empty or malformed request.
fn read_request<C: Read>(conn: &mut C) -> std::io::Result<Option<Request>> {
    let mut reader = BufReader::new(conn);

    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    let mut parts = line.split_whitespace();
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else {
        return Ok(None);
    };
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_owned(), query.to_owned()),
        None => (target.to_owned(), String::new()),
    };

    // Drain the headers, noting a body length so a POST can be consumed politely.
    let mut content_length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        if let Some(value) = header.strip_prefix("Content-Length:") {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }
    if content_length > 0 {
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body)?;
    }

    Ok(Some(Request {
        method: method.to_owned(),
        path,
        query,
    }))
}

// Pulls one value out of a `key=value&...` query string.
fn query_value(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == key).then(|| value.to_owned())
    })
}

fn snapshot_json<S: StateSource>(source: &Arc<Mutex<S>>) -> String {
    match source.lock() {
        Ok(mut source) => source
            .snapshot()
            .to_json()
            .unwrap_or_else(|_| "{}".to_owned()),
        Err(_) => "{}".to_owned(),
    }
}

// Streams a fresh snapshot as a server-sent event on a repeating cadence until the
// client disconnects, which surfaces as a write error.
fn stream_events<S: StateSource, W: Write>(
    conn: &mut W,
    source: &Arc<Mutex<S>>,
    interval: Duration,
) -> std::io::Result<()> {
    let headers = "HTTP/1.1 200 OK\r\n\
         Content-Type: text/event-stream\r\n\
         Cache-Control: no-cache\r\n\
         Connection: keep-alive\r\n\r\n";
    conn.write_all(headers.as_bytes())?;
    conn.flush()?;
    loop {
        let json = snapshot_json(source);
        if conn
            .write_all(format!("data: {json}\n\n").as_bytes())
            .is_err()
        {
            break;
        }
        if conn.flush().is_err() {
            break;
        }
        thread::sleep(interval);
    }
    Ok(())
}

fn write_response<W: Write>(
    conn: &mut W,
    code: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {code} {reason}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {len}\r\n\
         Connection: close\r\n\r\n",
        len = body.len(),
    );
    conn.write_all(header.as_bytes())?;
    conn.write_all(body)?;
    conn.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Mock, Scenario};

    #[test]
    fn query_value_extracts_a_named_parameter() {
        assert_eq!(
            query_value("scenario=alarm&locale=sw", "scenario").as_deref(),
            Some("alarm")
        );
        assert_eq!(query_value("locale=sw", "scenario"), None);
        assert_eq!(query_value("", "scenario"), None);
    }

    #[test]
    fn a_get_state_request_is_served_as_json_over_any_stream() {
        // The handler runs over an in-memory stream with no socket, proving it is
        // transport-agnostic: this is exactly what a TLS transport would plug into.
        let mut conn = MemConn::new(b"GET /state HTTP/1.1\r\nHost: x\r\n\r\n");
        let source = Arc::new(Mutex::new(Mock::new(Scenario::Alarm)));
        handle(
            &mut conn,
            source,
            Assets::Embedded,
            Duration::from_millis(0),
        )
        .expect("handled");
        let written = String::from_utf8_lossy(&conn.output);
        assert!(written.contains("200 OK"));
        assert!(written.contains("\"status\":\"alarm\""));
    }

    #[test]
    fn an_unknown_path_is_a_404() {
        let mut conn = MemConn::new(b"GET /nope HTTP/1.1\r\n\r\n");
        let source = Arc::new(Mutex::new(Mock::new(Scenario::Normal)));
        handle(
            &mut conn,
            source,
            Assets::Embedded,
            Duration::from_millis(0),
        )
        .expect("handled");
        assert!(String::from_utf8_lossy(&conn.output).contains("404 Not Found"));
    }

    // An in-memory connection: reads from a fixed request, collects the response.
    struct MemConn {
        input: std::io::Cursor<Vec<u8>>,
        output: Vec<u8>,
    }

    impl MemConn {
        fn new(request: &[u8]) -> Self {
            Self {
                input: std::io::Cursor::new(request.to_vec()),
                output: Vec::new(),
            }
        }
    }

    impl Read for MemConn {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.input.read(buf)
        }
    }

    impl Write for MemConn {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.output.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
