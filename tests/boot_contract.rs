#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("zed-sidecar-boot-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("create boot fixture");
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        for file in [".cli-flags.toml", ".ores-sidecar.toml"] {
            fs::copy(source.join(file), root.join(file))
                .unwrap_or_else(|error| panic!("copy {file}: {error}"));
        }
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn command(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_zed-sidecar"));
    command.current_dir(root);
    for key in [
        "ZED_SIDECAR_BIND",
        "ZED_SIDECAR_COMMAND",
        "ZED_SIDECAR_COMMAND_PREFLIGHT",
        "ZED_SIDECAR_COMMAND_PROBE",
        "ZED_SIDECAR_COMMAND_PROBE_READYZ",
        "ZED_SIDECAR_ALLOW_NON_LOOPBACK",
        "ORES_OTEL_SIDECAR_BIND",
        "ORES_OTEL_SIDECAR_ALLOW_NON_LOOPBACK",
        "FLAGS2ENV_DOTENV",
    ] {
        command.env_remove(key);
    }
    command
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn contract_keeps_cross_version_dotenv_isolation() {
    let source = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(".cli-flags.toml"))
        .expect("read flags contract");
    assert!(source.lines().any(|line| line.trim() == "dotenv = false"));
    assert!(source.lines().any(|line| line.trim() == "files = []"));
}

#[test]
fn preflight_accepts_the_repository_default_contracts() {
    let fixture = Fixture::new();
    let output = command(fixture.path())
        .arg("preflight")
        .output()
        .expect("run default preflight");
    assert_success(&output);
}

#[test]
fn argv_bind_overrides_process_environment_but_loopback_policy_stays_closed() {
    let fixture = Fixture::new();
    let output = command(fixture.path())
        .env("ZED_SIDECAR_BIND", "0.0.0.0:19090")
        .args(["preflight", "--bind=127.0.0.1:19091"])
        .output()
        .expect("run argv precedence preflight");
    assert_success(&output);
}

#[test]
fn caller_dotenv_is_not_an_implicit_configuration_source() {
    let fixture = Fixture::new();
    fs::write(
        fixture.path().join(".env"),
        "ZED_SIDECAR_BIND=0.0.0.0:19090\n",
    )
    .expect("write hostile dotenv fixture");

    let output = command(fixture.path())
        .arg("preflight")
        .output()
        .expect("run dotenv-isolation preflight");
    assert_success(&output);
}

#[test]
fn non_loopback_bind_fails_closed_without_reflecting_the_value() {
    let fixture = Fixture::new();
    let rejected = "0.0.0.0:19090";
    let output = command(fixture.path())
        .env("ZED_SIDECAR_BIND", rejected)
        .arg("preflight")
        .output()
        .expect("run non-loopback preflight");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains(rejected), "rejected bind leaked to stdout");
    assert!(!stderr.contains(rejected), "rejected bind leaked to stderr");
}

#[test]
fn unknown_argv_fails_at_the_cli_boundary() {
    let fixture = Fixture::new();
    let output = command(fixture.path())
        .args(["preflight", "--definitely-not-a-sidecar-option"])
        .output()
        .expect("run unknown-option case");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn missing_runtime_policy_fails_even_for_preflight() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.path().join(".ores-sidecar.toml")).expect("remove sidecar policy");
    let output = command(fixture.path())
        .arg("preflight")
        .output()
        .expect("run missing-policy preflight");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn missing_flags_contract_fails_at_the_cli_boundary() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.path().join(".cli-flags.toml")).expect("remove flags contract");
    let output = command(fixture.path())
        .arg("preflight")
        .output()
        .expect("run missing-flags preflight");
    assert_eq!(output.status.code(), Some(2));
}
