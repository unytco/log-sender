fn timestamp() -> String {
    std::time::SystemTime::UNIX_EPOCH
        .elapsed()
        .unwrap()
        .as_micros()
        .to_string()
}

#[derive(serde::Serialize)]
struct Start {
    k: &'static str,
    t: String,
}

impl Start {
    pub fn encode() -> String {
        let mut out = serde_json::to_string(&Start {
            k: "start",
            t: timestamp(),
        })
        .unwrap();
        out.push_str("\n");
        out
    }
}

#[derive(serde::Serialize)]
struct FetchedOps {
    k: &'static str,
    t: String,
    d: &'static str,
    c: &'static str,
    b: &'static str,
    a: &'static str,
    s: &'static str,
}

impl FetchedOps {
    pub fn encode() -> String {
        let mut out = serde_json::to_string(&FetchedOps {
            k: "fetchedOps",
            t: timestamp(),
            d: "bobo",
            c: "1",
            b: "42",
            a: "bobo",
            s: "bobo",
        })
        .unwrap();
        out.push_str("\n");
        out
    }
}

pub fn main() {
    use std::io::Write;

    let path = std::path::PathBuf::from(".");
    let mut file = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::MINUTELY)
        .max_log_files(3)
        .filename_prefix("hc-report")
        .filename_suffix("jsonl")
        .build(path)
        .unwrap();

    println!("running");

    file.write_all(Start::encode().as_bytes()).unwrap();

    loop {
        file.write_all(FetchedOps::encode().as_bytes()).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
