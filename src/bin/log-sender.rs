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
    },
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let arg: Arg = clap::Parser::parse();

    println!("Running Command: {:#?}", &arg.cmd);

    match arg.cmd {
        Cmd::Init {
            config_file,
            endpoint,
            unyt_pub_key,
        } => log_sender::initialize(config_file, endpoint, unyt_pub_key)
            .await
            .unwrap(),
    }

    /*
    let (pk, sk) = log_sender::crypto::generate_keypair().await.unwrap();

    let r = log_sender::client::Client::new(
        reqwest::Url::parse("http://127.0.0.1:8787").unwrap(),
    )
    .await
    .unwrap();

    r.health().await.unwrap();

    println!("server healthy");

    let id = r.drone_registration(&pk, &sk).await.unwrap();

    println!("registered device, got id: {id}");
    */
}
