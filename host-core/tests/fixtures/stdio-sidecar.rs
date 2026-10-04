use std::io::{self, BufRead, Write};

fn main() {
    let mode = std::env::args().nth(1).unwrap();
    if mode == "idle-exit" {
        std::process::exit(7);
    }
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let mut request_count = 0;
    if mode == "no-read" {
        loop {
            std::thread::park();
        }
    }
    for line in io::stdin().lock().lines() {
        assert!(line.unwrap().contains("runtime.greet"));
        request_count += 1;
        if mode == "buffered" {
            if request_count == 1 {
                println!("{{\"id\":\"test\",\"ok\":true,\"payload\":{{}}}}");
                println!("{{\"id\":\"next\",\"ok\":true,\"payload\":{{}}}}");
                io::stdout().flush().unwrap();
            }
            continue;
        }
        if mode == "crash" {
            std::process::exit(7);
        }
        if mode == "silent" {
            loop {
                std::thread::park();
            }
        }
        if mode == "stderr" {
            io::stderr()
                .write_all(&vec![b'x'; 2 * 1024 * 1024])
                .unwrap();
        }
        let response = match mode.as_str() {
            "lifecycle" => format!(
                "{{\"id\":\"test\",\"ok\":true,\"payload\":{{\"port\":{}}}}}",
                listener.local_addr().unwrap().port()
            ),
            "bad-shape" => r#"{"id":"test","ok":"yes","payload":{}}"#.to_owned(),
            "missing-payload" => r#"{"id":"test","ok":true}"#.to_owned(),
            "application-error" => {
                r#"{"id":"test","ok":false,"payload":{"error":"unknown command"}}"#.to_owned()
            }
            "wrong-id" => r#"{"id":"other","ok":true,"payload":{}}"#.to_owned(),
            "malformed" => "not JSON".to_owned(),
            "oversized" => "x".repeat(1024 * 1024 + 1),
            "truncated" => {
                print!("{{\"id\":");
                return;
            }
            _ => r#"{"id":"test","ok":true,"payload":{"message":"hello"}}"#.to_owned(),
        };
        println!("{response}");
        io::stdout().flush().unwrap();
    }
}
