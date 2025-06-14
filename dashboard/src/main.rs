use clap::{Arg, Command};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = Command::new("dashboard")
        .about("BCAI Network Dashboard - Web interface for monitoring jobs and network status")
        .version("0.1.0")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to bind the dashboard server")
                .default_value("8000"),
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("Host address to bind the dashboard server")
                .default_value("127.0.0.1"),
        )
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap();
    let port = matches.get_one::<String>("port").unwrap();
    let addr = format!("{}:{}", host, port);

    println!("Starting BCAI Dashboard on {}", addr);
    println!("Visit http://{}/jobs to view active jobs", addr);

    dashboard::serve(&addr)
}
