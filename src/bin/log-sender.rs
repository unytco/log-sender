#[tokio::main(flavor = "multi_thread")]
async fn main() {
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
}
