#![forbid(unsafe_code)]

// Source-contract test: executable preflight is already exercised by
// tests/boot_contract.rs, while this fixes the deployment wiring.
#[test]
fn k8s_startup_probe_runs_config_preflight_before_liveness() {
    let manifest = include_str!("../k8s/container.yaml");
    assert!(
        manifest.contains(
            "startupProbe:\n    exec:\n      command: [\"/zed-sidecar\", \"preflight\"]"
        ),
        "startup probe must use the product's preflight contract"
    );
    assert!(
        manifest.contains(
            "livenessProbe:\n    exec:\n      command: [\"/zed-sidecar\", \"probe\"]"
        ),
        "liveness must keep the bounded local health probe"
    );
    assert!(
        !manifest.lines().any(|line| {
            matches!(line.trim(), "readinessProbe:" | "httpGet:" | "tcpSocket:")
        }),
        "sidecar must not expose Kubernetes network probes"
    );
    assert!(manifest.contains("runAsNonRoot: true"));
    assert!(manifest.contains("readOnlyRootFilesystem: true"));
    assert!(manifest.contains("value: \"127.0.0.1:9090\""));
}
