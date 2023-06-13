use log::{error, info, LevelFilter};
use pass_it_on::start_server;
use pass_it_on::Error;
use pass_it_on::ServerConfiguration;
use std::path::PathBuf;

const LOG_TARGET: &str = "pass_it_on_server";

#[tokio::main]
async fn main() {
    let module_log_level = module_debug_level();
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .env()
        .with_module_level(pass_it_on::LIB_LOG_TARGET, module_log_level)
        .with_module_level(LOG_TARGET, module_log_level)
        .with_colors(true)
        .init()
        .unwrap();

    if let Err(error) = run().await {
        error!(target: LOG_TARGET, "{}", error)
    }
}
async fn run() -> Result<(), Error> {
    // Setup default directories
    let default_config_path = directories::ProjectDirs::from("net", "pass-on", "pass-on").unwrap();

    // Parse Config file
    let server_config = {
        let args: Vec<String> = std::env::args().collect();
        let config_path = match args.get(1) {
            Some(path) => PathBuf::from(path),
            None => PathBuf::from(default_config_path.config_dir()).join("server.toml"),
        };

        info!(target: LOG_TARGET, "Reading configuration from: {}", config_path.to_str().unwrap());
        ServerConfiguration::from_toml(std::fs::read_to_string(config_path)?.as_str())?
    };

    start_server(server_config, None).await
}

fn module_debug_level() -> LevelFilter {
    let args: Vec<String> = std::env::args().collect();
    let limit = args.len() - 1;

    for i in 0..limit {
        if args.get(i).unwrap().contains("--log-level") && i < limit {
            match args.get(i + 1).unwrap().trim().to_ascii_lowercase().as_str() {
                "trace" => return LevelFilter::Trace,
                "debug" => return LevelFilter::Debug,
                "info" => return LevelFilter::Info,
                "warn" => return LevelFilter::Warn,
                "error" => return LevelFilter::Error,
                _ => (),
            }
        }
    }
    LevelFilter::Info
}
