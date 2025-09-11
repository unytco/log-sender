use crate::config::*;

#[tokio::test(flavor = "multi_thread")]
async fn config_init() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("runtime-config.json");
    let c = RuntimeConfigFile::with_init(
        file.clone(),
        "http://127.0.0.1:8787".into(),
        "bla".into(),
        42,
    )
    .await
    .unwrap();

    // manually read the file
    let data = tokio::fs::read_to_string(c.path()).await.unwrap();
    let data: RuntimeConfig = serde_json::from_str(&data).unwrap();
    println!("{data:#?}");

    assert_eq!(&*c, &data);
}
