use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

#[test]
fn one_command_writes_all_three_sizing_runs() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after Unix epoch")
        .as_nanos();
    let output_path = std::env::temp_dir().join(format!(
        "wss-mux-sizing-integration-{}-{nonce}.json",
        std::process::id()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_wss-mux-loadgen"))
        .args([
            "--duration",
            "1",
            "sizing-run",
            "--instance-label",
            "integration-test",
            "--connection-start",
            "2",
            "--connection-max",
            "2",
            "--connection-event-rate",
            "10",
            "--fixed-connections",
            "2",
            "--event-rate-start",
            "10",
            "--event-rate-max",
            "10",
            "--slow-every",
            "2",
            "--memory-payload-bytes",
            "128",
            "--out",
        ])
        .arg(&output_path)
        .output()
        .expect("run sizing command");
    assert!(
        output.status.success(),
        "sizing command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let document: Value =
        serde_json::from_slice(&fs::read(&output_path).expect("read sizing output"))
            .expect("parse sizing output");
    assert_eq!(document["instance_label"], "integration-test");
    assert_eq!(document["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(document["commit"].as_str().map(str::len), Some(40));
    assert!(document["dirty"].is_boolean());
    assert!(matches!(
        document["build_profile"].as_str(),
        Some("debug" | "release")
    ));
    assert_eq!(document["provenance_source"], "runtime_checkout");
    assert!(document["date"]
        .as_str()
        .is_some_and(|date| date.ends_with('Z')));
    assert_eq!(
        document["methodology"]["resource_process"],
        "isolated_server"
    );
    let runs = document["runs"].as_array().expect("runs array");
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[0]["name"], "connections");
    assert_eq!(runs[1]["name"], "throughput");
    assert_eq!(runs[2]["name"], "memory");
    for run in runs {
        assert!(run["saturation_point"].is_u64());
        assert!(run["rss"].is_string());
        assert!(run["cpu"].is_string());
        assert!(run["recommended_requests"].is_object());
        assert!(run["recommended_limits"].is_object());
    }

    fs::remove_file(output_path).expect("remove sizing output");
}
