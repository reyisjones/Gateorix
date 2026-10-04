use gateorix_adapter::GateorixAdapter;
use serde_json::json;

fn main() -> std::io::Result<()> {
    let mut adapter = GateorixAdapter::new();
    adapter.command("greet", |payload| {
        let name = payload
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("World");
        Ok(json!({"message": format!("Hello, {name}! Welcome to Gateorix Rust.")}))
    });
    adapter.command("echo", Ok);
    if std::env::args().any(|argument| argument == "--http") {
        adapter.run_http(3001)
    } else {
        adapter.run()
    }
}
