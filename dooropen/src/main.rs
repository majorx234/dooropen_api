use clap::{App, Arg};
use dooropen_lib::server;

#[tokio::main]
async fn main() {
    env_logger::init();

    let matches = App::new("server")
        .arg(
            Arg::with_name("https")
                .long("https")
                .help("Whether to use HTTPS or not"),
        )
        .get_matches();

    let addr = "127.0.0.1:8080";

    server::create(addr, matches.is_present("https")).await;
}
