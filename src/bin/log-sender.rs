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

        /// Specify one or more paths to directories that will contain log files
        /// with entries to be published as log-collector metrics. The sender
        /// will parse all files ending in a `.jsonl` extension. Specify
        /// this argument multiple times on the command line, or if using
        /// an environment variable, separate the paths with commas.
        #[arg(long, env = "LOG_SENDER_REPORT_PATHS", value_delimiter = ',')]
        report_path: Vec<std::path::PathBuf>,

        /// Specify one or more conductor config paths. These will be used
        /// to report on database sizes on-disk at the reporting interval.
        /// Specify this argument multiple times on the command line, or if
        /// using an environment variable, separate the paths with commas.
        #[arg(
            long,
            env = "LOG_SENDER_CONDUCTOR_CONFIG_PATHS",
            value_delimiter = ','
        )]
        conductor_config_path: Vec<std::path::PathBuf>,
    },

    /// Register DNA hashes with agreements and optional price sheets for a
    /// drone.
    RegisterDna {
        /// Specify a full path to a config file,
        /// e.g. `/var/run/log-sender-runtime.json`.
        #[arg(long, env = "LOG_SENDER_CONFIG_FILE")]
        config_file: std::path::PathBuf,

        /// The dna hash to register.
        #[arg(long, env = "LOG_SENDER_DNA_HASH")]
        dna_hash: String,

        /// The agreement id to register.
        #[arg(long, env = "LOG_SENDER_AGREEMENT_ID")]
        agreement_id: String,

        /// Optionally attach a price-sheet hash.
        #[arg(long, env = "LOG_SENDER_PRICE_SHEET_HASH")]
        price_sheet_hash: Option<String>,

        /// Optionally include additional json metadata.
        #[arg(long, env = "LOG_SENDER_METADATA")]
        metadata: Option<String>,
    },

    /// Run the service, polling a log-file directory for metrics to
    /// publish to the log-collector.
    Service {
        /// Specify a full path to a config file,
        /// e.g. `/var/run/log-sender-runtime.json`.
        #[arg(long, env = "LOG_SENDER_CONFIG_FILE")]
        config_file: std::path::PathBuf,
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
            report_path,
            conductor_config_path,
        } => log_sender::initialize(
            config_file,
            endpoint,
            unyt_pub_key,
            report_interval_seconds,
            report_path,
            conductor_config_path,
        )
        .await
        .unwrap(),
        Cmd::RegisterDna {
            config_file,
            dna_hash,
            agreement_id,
            price_sheet_hash,
            metadata,
        } => {
            let out = log_sender::register_dna(
                config_file,
                dna_hash,
                agreement_id,
                price_sheet_hash,
                metadata.map(|s| serde_json::from_str(&s).unwrap()),
            )
            .await
            .unwrap();
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        Cmd::Service { config_file } => {
            log_sender::run_service(config_file).await.unwrap()
        }
    }
}
