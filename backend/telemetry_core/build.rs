use std::process::Command;

fn main() {
    // Fetch the git hash if possible, <unknown> if not.
    let git_hash = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .map(|output| String::from_utf8(output.stdout).unwrap_or_default())
        .unwrap_or_default();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}