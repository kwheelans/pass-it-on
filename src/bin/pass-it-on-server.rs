use clap::Parser;
use tracing::{error, info};
use pass_it_on::Error;
use pass_it_on::ServerConfiguration;
use pass_it_on::{start_server, verify_matrix_devices};
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::level_filters::LevelFilter;

#[derive(Parser, Debug)]
#[clap(name = "pass-it-on-server", author, version, about = "Pass-it-on server binary", long_about = None)]
struct CliArgs {
    /// Path to pass-it-on server configuration file
    #[clap(short, long, value_parser)]
    configuration: Option<PathBuf>,
    /// Set the logging level [default: Info]
    #[clap(short, long, value_parser)]
    log_level: Option<LevelFilter>,
    /// Interactively verify Matrix endpoint devices when set
    #[clap(short, long, value_parser, default_value_t = false)]
    matrix_verify_devices: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = CliArgs::parse();
    tracing_subscriber::fmt().with_max_level(cli.log_level.unwrap_or(LevelFilter::INFO)).init();

    match run(cli).await {
        Err(error) => {
            error!("{}", error);
            ExitCode::FAILURE
        }
        Ok(_) => ExitCode::SUCCESS,
    }
}

async fn run(cliargs: CliArgs) -> Result<(), Error> {
    info!("Log level is set to {}", cliargs.log_level.unwrap_or(LevelFilter::INFO));
    // Setup default directories
    let default_config_path = directories::ProjectDirs::from("com", "pass-it-on", "pass-it-on-server").unwrap();

    // Parse Config file
    let server_config = {
        let config_path = match cliargs.configuration {
            Some(path) => path,
            None => PathBuf::from(default_config_path.config_dir()).join("server.toml"),
        };

        info!("Reading configuration from: {}", config_path.to_str().unwrap());
        ServerConfiguration::try_from(std::fs::read_to_string(config_path)?.as_str())?
    };

    // Run interactive matrix device verification when flag is passed
    match cliargs.matrix_verify_devices {
        true => verify_matrix_devices(server_config).await,
        false => start_server(server_config, None, None).await,
    }
}
