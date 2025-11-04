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
        60,
        vec![dir.path().into()],
        vec![dir.path().join("conductor-config.yaml")],
    )
    .await
    .unwrap();

    let path = c.path().to_owned();
    let c: RuntimeConfig = c.into();

    // manually read the file
    // (windows sometimes takes a bit to clean up the lock)
    let data = async {
        for _ in 0..10 {
            if let Ok(data) = tokio::fs::read_to_string(&path).await {
                return data;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        panic!("could not read config file");
    }
    .await;

    let data: RuntimeConfig = serde_json::from_str(&data).unwrap();

    println!("{data:#?}");

    assert_eq!(&c, &data);
}
