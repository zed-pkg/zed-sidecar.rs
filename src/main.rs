#![forbid(unsafe_code)]

#[path = "../generated/rust/env.rs"]
mod env;

use ores_otel_sidecar::{
    cli, preflight_startup_with_keys, runtime, SidecarConfig, SidecarIdentity,
};

const CLI_CONFIG_PATH: &str = ".cli-flags.toml";
const ALLOW_NON_LOOPBACK_ENV: &str = "ZED_SIDECAR_ALLOW_NON_LOOPBACK";

fn main() {
    let identity = SidecarIdentity::new(env::SERVICE, env::BIND);
    let invocation = match cli::resolve_process(CLI_CONFIG_PATH) {
        Ok(invocation) => invocation,
        Err(_) => runtime::exit_invalid_cli(identity),
    };
    let command = invocation.command;

    // Resolve all defaults/env/argv through flags-2-env and type-check the
    // immutable snapshot before any listener or runtime side effect exists.
    let startup = match preflight_startup_with_keys(
        invocation.values(),
        env::BIND,
        ALLOW_NON_LOOPBACK_ENV,
    ) {
        Ok(startup) => startup,
        Err(_) => runtime::exit_invalid_config(identity),
    };

    // zed-sidecar intentionally remains loopback-only even though the shared
    // parent supports an explicit non-loopback escape hatch for other products.
    if startup.allow_non_loopback {
        runtime::exit_invalid_config(identity);
    }

    let cfg = match SidecarConfig::from_bind(identity, &startup.bind, false) {
        Ok(cfg) => cfg,
        Err(_) => runtime::exit_invalid_config(identity),
    };

    runtime::run_command(&cfg, command);
}
