#![forbid(unsafe_code)]

use ores_otel_sidecar::{
    cli, preflight_loopback_only_with_key, runtime, SidecarConfig, SidecarIdentity,
    DEFAULT_SIDECAR_CONFIG_PATH,
};

const CLI_CONFIG_PATH: &str = ".cli-flags.toml";
const BIND_ENV: &str = "ZED_SIDECAR_BIND";
const SERVICE: &str = env!("CARGO_PKG_NAME");

fn main() {
    let identity = SidecarIdentity::new(SERVICE, BIND_ENV);
    let invocation = match cli::resolve_process(CLI_CONFIG_PATH) {
        Ok(invocation) => invocation,
        Err(_) => runtime::exit_invalid_cli(identity),
    };
    let command = invocation.command;

    // Materialize defaults/env/argv through the one flags-2-env authority, then
    // type-check the product-owned bind key before any listener/runtime effect.
    let startup = match preflight_loopback_only_with_key(invocation.values(), BIND_ENV) {
        Ok(startup) => startup,
        Err(_) => runtime::exit_invalid_config(identity),
    };

    let cfg = match SidecarConfig::from_bind(identity, &startup.bind, false) {
        Ok(cfg) => cfg,
        Err(_) => runtime::exit_invalid_config(identity),
    };

    // The independently TJSV-admitted .ores-sidecar.toml is also a runtime
    // admission gate. Product sidecars must not validate a policy file in CI
    // and then silently ignore that same policy in the executable.
    let cfg = match cfg.with_sidecar_file(DEFAULT_SIDECAR_CONFIG_PATH) {
        Ok(cfg) => cfg,
        Err(_) => runtime::exit_invalid_config(identity),
    };

    runtime::run_command(&cfg, command);
}
