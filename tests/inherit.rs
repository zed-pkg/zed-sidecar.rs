#![forbid(unsafe_code)]

use ores_otel_sidecar::{health, SidecarIdentity};

const SERVICE: &str = env!("CARGO_PKG_NAME");
const BIND_ENV: &str = "ZED_SIDECAR_BIND";

#[test]
fn inherits_shared_health_with_product_identity() {
    let identity = SidecarIdentity::new(SERVICE, BIND_ENV);
    let payload = health::current(identity, None);
    assert!(payload.ok);
    assert_eq!(payload.service, SERVICE);
}

#[test]
fn repository_contract_is_loopback_only_and_dotenv_closed() {
    let flags = include_str!("../.cli-flags.toml");
    assert!(flags.contains("env = \"ZED_SIDECAR_BIND\""));
    assert!(flags.contains("files = []"));
    assert!(flags.contains("allow_unknown = false"));
    assert!(!flags.contains("allow-non-loopback"));
    assert!(!flags.contains("ZED_SIDECAR_ALLOW_NON_LOOPBACK"));
    assert!(!flags.contains("[identity]"));

    let sidecar = include_str!("../.ores-sidecar.toml");
    assert!(sidecar.contains("id = \"zed-sidecar\""));
    assert!(sidecar.contains("startup_config = \".cli-flags.toml\""));
    assert!(sidecar.contains("mode = \"disabled\""));
}
