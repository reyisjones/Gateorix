use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use tiny_http::{Header, Method, Response, Server};

const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
const DEV_ORIGIN: &str = "http://localhost:5173";
type Handler = Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>;

#[derive(Deserialize)]
struct Request {
    id: String,
    channel: String,
    payload: Value,
}

#[derive(Default)]
pub struct GateorixAdapter {
    handlers: HashMap<String, Handler>,
}

impl GateorixAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn command(
        &mut self,
        name: &str,
        handler: impl Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
    ) {
        self.handlers.insert(name.to_owned(), Box::new(handler));
    }

    pub fn dispatch(&self, bytes: &[u8]) -> Value {
        if bytes.len() > MAX_MESSAGE_BYTES {
            return failure("", "message exceeds 1 MiB");
        }
        let request: Request = match serde_json::from_slice(bytes) {
            Ok(request) => request,
            Err(_) => return failure("", "invalid request envelope"),
        };
        if request.id.is_empty() || !request.payload.is_object() {
            return failure(
                &request.id,
                "id must be nonempty and payload must be an object",
            );
        }
        let action = request
            .channel
            .strip_prefix("runtime.")
            .unwrap_or(&request.channel);
        if action.contains('.') {
            return failure(&request.id, "unknown runtime namespace");
        }
        match self.handlers.get(action) {
            Some(handler) => match handler(request.payload) {
                Ok(payload) => json!({"id": request.id, "ok": true, "payload": payload}),
                Err(error) => failure(&request.id, &error),
            },
            None => failure(&request.id, "unknown command"),
        }
    }

    pub fn run(&self) -> io::Result<()> {
        self.run_stdio(io::stdin().lock(), io::stdout().lock())
    }

    pub fn run_stdio(&self, mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
        loop {
            let mut frame = Vec::new();
            let size = (&mut input)
                .take((MAX_MESSAGE_BYTES + 1) as u64)
                .read_until(b'\n', &mut frame)?;
            if size == 0 {
                return Ok(());
            }
            if size > MAX_MESSAGE_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "message exceeds 1 MiB",
                ));
            }
            if frame.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            serde_json::to_writer(&mut output, &self.dispatch(&frame))?;
            output.write_all(b"\n")?;
            output.flush()?;
        }
    }

    pub fn run_http(&self, port: u16) -> io::Result<()> {
        let server =
            Server::http((std::net::Ipv4Addr::LOCALHOST, port)).map_err(io::Error::other)?;
        eprintln!("Gateorix Rust development adapter: http://127.0.0.1:{port}");
        for mut request in server.incoming_requests() {
            let header = |name: &str| {
                request
                    .headers()
                    .iter()
                    .find(|header| header.field.as_str().as_str().eq_ignore_ascii_case(name))
                    .map(|header| header.value.as_str())
            };
            let origin_allowed = header("origin").is_none_or(|origin| origin == DEV_ORIGIN);
            let host_allowed = header("host").is_some_and(|host| {
                host == format!("localhost:{port}") || host == format!("127.0.0.1:{port}")
            });
            let content_type_allowed = header("content-type").is_some_and(|value| {
                value.split(';').next().unwrap_or("").trim() == "application/json"
            });
            let (status, payload) = if !origin_allowed || !host_allowed {
                (403, failure("", "origin or host denied"))
            } else if request.url() == "/health" && request.method() == &Method::Get {
                (200, json!({"status": "ok"}))
            } else if request.url() != "/invoke" {
                (404, failure("", "not found"))
            } else if request.method() == &Method::Options {
                (204, Value::Null)
            } else if request.method() != &Method::Post {
                (405, failure("", "method not allowed"))
            } else if !content_type_allowed {
                (415, failure("", "application/json required"))
            } else if request
                .body_length()
                .is_none_or(|length| length > MAX_MESSAGE_BYTES)
            {
                (413, failure("", "bounded Content-Length required"))
            } else {
                let mut body = Vec::new();
                request
                    .as_reader()
                    .take((MAX_MESSAGE_BYTES + 1) as u64)
                    .read_to_end(&mut body)?;
                if body.len() > MAX_MESSAGE_BYTES {
                    (413, failure("", "message exceeds 1 MiB"))
                } else {
                    (200, self.dispatch(&body))
                }
            };
            let mut response = Response::from_string(payload.to_string()).with_status_code(status);
            for (name, value) in [
                ("Content-Type", "application/json"),
                ("Access-Control-Allow-Origin", DEV_ORIGIN),
                ("Access-Control-Allow-Methods", "POST, OPTIONS"),
                ("Access-Control-Allow-Headers", "Content-Type"),
            ] {
                response.add_header(Header::from_bytes(name, value).expect("static HTTP header"));
            }
            request.respond(response)?;
        }
        Ok(())
    }
}

fn failure(id: &str, message: &str) -> Value {
    json!({"id": id, "ok": false, "payload": {"error": message}})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdio_correlates_requests_and_recovers_from_malformed_frames() {
        let mut adapter = GateorixAdapter::new();
        adapter.command("echo", Ok);
        let input = b"[]\n{\"id\":\"one\",\"channel\":\"runtime.echo\",\"payload\":{\"hello\":true}}\n{\"id\":\"two\",\"channel\":\"filesystem.echo\",\"payload\":{}}\n";
        let mut output = Vec::new();
        adapter.run_stdio(&input[..], &mut output).unwrap();
        let responses: Vec<Value> = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(responses.len(), 3);
        assert_eq!(responses[0]["ok"], false);
        assert_eq!(responses[1]["id"], "one");
        assert_eq!(responses[1]["payload"]["hello"], true);
        assert_eq!(responses[2]["id"], "two");
        assert_eq!(responses[2]["ok"], false);
    }

    #[test]
    fn rejects_invalid_envelopes_and_oversized_frames() {
        let adapter = GateorixAdapter::new();
        for frame in [
            "null",
            "{}",
            r#"{"id":"x","channel":"greet","payload":null}"#,
        ] {
            assert_eq!(adapter.dispatch(frame.as_bytes())["ok"], false);
        }
        let oversized = vec![b'x'; MAX_MESSAGE_BYTES + 1];
        assert!(adapter.run_stdio(&oversized[..], Vec::new()).is_err());
    }
}
