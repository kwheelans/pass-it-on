use clap::Parser;
use log::{error, info, LevelFilter};
use pass_it_on::start_server;
use pass_it_on::Error;
use pass_it_on::ServerConfiguration;
use std::path::PathBuf;

const LOG_TARGET: &str = "pass_it_on_server";

#[derive(Parser, Debug)]
#[clap(name = "pass-it-on-server", author, version, about = "Pass-it-on server binary", long_about = None)]
struct CliArgs {
    /// Path to pass-it-on server configuration file
    #[clap(short, long, value_parser)]
    configuration: Option<PathBuf>,
    /// Set the logging level [default: Info]
    #[clap(short, long, value_parser)]
    log_level: Option<LevelFilter>,
}

#[tokio::main]
async fn main() {
    let cli = CliArgs::parse();
    let module_log_level = cli.log_level.unwrap_or(LevelFilter::Info);
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .env()
        .with_module_level(pass_it_on::LIB_LOG_TARGET, module_log_level)
        .with_module_level(LOG_TARGET, module_log_level)
        .with_colors(true)
        .init()
        .unwrap();

    if let Err(error) = run(cli).await {
        error!(target: LOG_TARGET, "{}", error)
    }
}
async fn run(args: CliArgs) -> Result<(), Error> {
    // Setup default directories
    let default_config_path = directories::ProjectDirs::from("com", "pass-it-on", "pass-it-on-server").unwrap();

    // Parse Config file
    let server_config = {
        let config_path = match args.configuration {
            Some(path) => path,
            None => PathBuf::from(default_config_path.config_dir()).join("server.toml"),
        };

        info!(target: LOG_TARGET, "Reading configuration from: {}", config_path.to_str().unwrap());
        ServerConfiguration::try_from(std::fs::read_to_string(config_path)?.as_str())?
    };

    start_server(server_config, None).await
}
