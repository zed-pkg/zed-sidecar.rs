#![forbid(unsafe_code)]

#[path = "../generated/rust/env.rs"]
mod env;

use ores_otel_sidecar::{runtime, SidecarConfig, SidecarIdentity};
use ores_sidecar::SidecarConfig as SharedSidecarConfig;

fn selected_bind(sidecar: &ores_sidecar::Sidecar, env_override: Option<String>) -> String {
    env_override
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| sidecar.bind_address())
}

fn main() {
    let identity = SidecarIdentity::new(env::SERVICE, env::BIND);
    let shared = match SharedSidecarConfig::load(ores_sidecar::CONFIG_FILE_NAME)
        .and_then(|config| config.resolve_one(Some(env::SERVICE)))
    {
        Ok(sidecar) if sidecar.enabled => sidecar,
        Ok(_) | Err(_) => runtime::exit_invalid_cli(identity),
    };
    let bind = selected_bind(&shared, std::env::var(env::BIND).ok());
    let cfg = match SidecarConfig::from_bind(identity, &bind, false) {
        Ok(cfg) => cfg,
        Err(_) => runtime::exit_invalid_cli(identity),
    };
    runtime::run(&cfg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_manifest_matches_generated_bind_default() {
        let config = SharedSidecarConfig::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(ores_sidecar::CONFIG_FILE_NAME),
        )
        .expect("load .ores-sidecar.toml");
        let sidecar = config
            .resolve_one(Some(env::SERVICE))
            .expect("resolve this sidecar");
        assert_eq!(sidecar.bind_address(), env::BIND_DEFAULT);
    }

    #[test]
    fn explicit_environment_bind_overrides_shared_default() {
        let config = SharedSidecarConfig::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(ores_sidecar::CONFIG_FILE_NAME),
        )
        .expect("load .ores-sidecar.toml");
        let sidecar = config
            .resolve_one(Some(env::SERVICE))
            .expect("resolve this sidecar");
        assert_eq!(
            selected_bind(&sidecar, Some("127.0.0.1:19090".to_owned())),
            "127.0.0.1:19090"
        );
        assert_eq!(selected_bind(&sidecar, Some("   ".to_owned())), env::BIND_DEFAULT);
    }
}
