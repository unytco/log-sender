#[derive(Debug, clap::Parser)]
#[command(version, about)]
struct Arg {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, clap::Subcommand)]
enum Cmd {
    /// Initialize a new config file. Note, this will generate a new
    /// drone cryptographic keypair, register it with the provided
    /// endpoint server, and write out a config file for operations.
    Init {
        /// Specify a full path to a config file,
        /// e.g. `/var/run/log-sender-runtime.json`.
        #[arg(long, env = "LOG_SENDER_CONFIG_FILE")]
        config_file: std::path::PathBuf,

        /// Specify the endpoint url of the log-collector endpoint,
        /// e.g. `https://log-collector.my.url`.
        #[arg(long, env = "LOG_SENDER_ENDPOINT")]
        endpoint: String,

        /// Base64 Unyt Public Key for registration.
        #[arg(long, env = "LOG_SENDER_UNYT_PUB_KEY")]
        unyt_pub_key: String,

        /// Frequency at which to run reporting.
        #[arg(long, env = "LOG_SENDER_REPORT_INTERVAL_SECONDS")]
        report_interval_seconds: u64,
    },

    /// Run the service, polling a log-file directory for metrics to
    /// publish to the log-collector.
    Service {
        /// Specify a full path to a config file,
        /// e.g. `/var/run/log-sender-runtime.json`.
        #[arg(long, env = "LOG_SENDER_CONFIG_FILE")]
        config_file: std::path::PathBuf,

        /// Specify a path to a directory that will contain log files
        /// with entries to be published as log-collector metrics.
        #[arg(long, env = "LOG_SENDER_REPORT_DIRECTORY")]
        report_directory: std::path::PathBuf,
    },
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(
                        tracing_subscriber::filter::LevelFilter::INFO.into(),
                    )
                    .from_env_lossy(),
            )
            .compact()
            .without_time()
            .finish(),
    )
    .unwrap();

    let arg: Arg = clap::Parser::parse();

    tracing::info!(cmd = ?arg.cmd, "Running Command");

    match arg.cmd {
        Cmd::Init {
            config_file,
            endpoint,
            unyt_pub_key,
            report_interval_seconds,
        } => log_sender::initialize(
            config_file,
            endpoint,
            unyt_pub_key,
            report_interval_seconds,
        )
        .await
        .unwrap(),
        Cmd::Service {
            config_file,
            report_directory,
        } => log_sender::run_service(config_file, report_directory)
            .await
            .unwrap(),
    }
}
