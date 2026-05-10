use std::process::Command;

fn main() {
    // Embed the git commit SHA
    let commit_sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_default();
    println!(
        "cargo:rustc-env=ALEPH_COMMIT_SHA={}",
        commit_sha.trim()
    );

    // Find the Zed base version from the most recent v*.*.* tag reachable from HEAD
    let base_version = Command::new("git")
        .args(["describe", "--tags", "--match", "v[0-9]*.[0-9]*.[0-9]*", "--abbrev=0"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_default();
    println!(
        "cargo:rustc-env=ALEPH_ZED_BASE_VERSION={}",
        base_version.trim().trim_start_matches('v')
    );

    // Only re-run if git HEAD changes
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs");
}
