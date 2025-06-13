fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dashboard::serve("127.0.0.1:8000")
}
