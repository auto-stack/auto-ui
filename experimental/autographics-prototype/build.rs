//! Embeds the AGP git commit into the binary for report fingerprints
//! (review r1 F08). Re-runs when HEAD moves; falls back to "unknown"
//! outside a git checkout. A "-dirty" suffix marks uncommitted changes so a
//! fingerprint can never silently claim a clean tree.

fn main() {
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false);
    // re-run when HEAD moves (worktree-safe: resolve the real gitdir)
    if let Ok(gitdir) = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    {
        let head = std::path::Path::new(&gitdir).join("HEAD");
        if head.exists() {
            println!("cargo:rerun-if-changed={}", head.display());
        }
        println!("cargo:rerun-if-changed={gitdir}/index");
    }
    let suffix = if dirty { "-dirty" } else { "" };
    println!("cargo:rustc-env=AGP_GIT_COMMIT={commit}{suffix}");
    println!("cargo:rerun-if-changed=build.rs");
}
