use std::process::Command;

#[test]
fn cli_help_succeeds() {
    // Use the built binary from cargo to ensure the CLI is runnable end-to-end.
    let output = Command::new(env!("CARGO_BIN_EXE_quicssh-rs-robust"))
        .arg("--help")
        .output()
        .expect("failed to spawn quicssh-rs-robust");

    assert!(
        output.status.success(),
        "cli --help exited with status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage: quicssh-rs-robust"),
        "unexpected help output: {}",
        stdout
    );
}
