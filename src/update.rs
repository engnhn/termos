use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const REPOSITORY: &str = "engnhn/termos";
const ASSET: &str = "termos-linux-x86_64";
const SIGNER_WORKFLOW: &str = "engnhn/termos/.github/workflows/release.yml";

fn validate_tag(tag: &str) -> Result<(), String> {
    if tag.is_empty()
        || !tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err("The latest release returned an invalid tag name".to_string());
    }
    Ok(())
}

fn validate_gh_version(output: &str) -> Result<(), String> {
    let version = output
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(2))
        .ok_or_else(|| "Could not determine GitHub CLI version".to_string())?;
    let mut parts = version.split('.').map(|part| part.parse::<u64>());
    let parsed = (
        parts.next().transpose(),
        parts.next().transpose(),
        parts.next().transpose(),
    );
    let (Ok(Some(major)), Ok(Some(minor)), Ok(Some(patch))) = parsed else {
        return Err("Could not parse GitHub CLI version".to_string());
    };
    if (major, minor, patch) < (2, 102, 0) {
        return Err(
            "GitHub CLI 2.102.0 or newer is required for safe attestation verification".to_string(),
        );
    }
    Ok(())
}

fn run(command: &mut Command, action: &str) -> Result<Output, String> {
    let output = command
        .output()
        .map_err(|e| format!("Could not {action}: {e}. Install GitHub CLI (`gh`) and retry."))?;
    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if details.is_empty() {
            format!("Could not {action}: command exited with {}", output.status)
        } else {
            format!("Could not {action}: {details}")
        });
    }
    Ok(output)
}

fn latest_tag() -> Result<String, String> {
    let output = run(
        Command::new("gh").args([
            "release", "view", "--repo", REPOSITORY, "--json", "tagName", "--jq", ".tagName",
        ]),
        "find the latest release",
    )?;
    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
    validate_tag(&tag)?;
    Ok(tag)
}

fn make_private_temp_dir() -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {e}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("termos-update-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&path)
        .map_err(|e| format!("Could not create private update directory: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("Could not secure update directory: {e}"))?;
    }
    Ok(path)
}

fn install_verified(binary: &Path, target: &Path) -> Result<(), String> {
    use std::io;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    let parent = target
        .parent()
        .ok_or_else(|| "Installed binary path has no parent directory".to_string())?;
    let mut staging = None;
    for _ in 0..100 {
        let candidate = parent.join(format!(
            ".termos-update-{}-{}",
            std::process::id(),
            NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&candidate) {
            Ok(mut file) => {
                let mut source = std::fs::File::open(binary)
                    .map_err(|e| format!("Could not open verified release binary: {e}"))?;
                std::io::copy(&mut source, &mut file)
                    .map_err(|e| format!("Could not stage verified release binary: {e}"))?;
                file.sync_all()
                    .map_err(|e| format!("Could not sync staged binary: {e}"))?;
                staging = Some(candidate);
                break;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => break,
            Err(e) => return Err(format!("Could not stage release binary: {e}")),
        }
    }

    if let Some(staging) = staging {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&staging, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Could not mark staged binary executable: {e}"))?;
        }
        match std::fs::rename(&staging, target) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                let _ = std::fs::remove_file(&staging);
            }
            Err(e) => {
                let _ = std::fs::remove_file(&staging);
                return Err(format!(
                    "Could not atomically replace installed binary: {e}"
                ));
            }
        }
    }

    let nonce = uuid::Uuid::new_v4();
    let privileged_staging = parent.join(format!(
        ".termos-update-{}-{nonce}-privileged",
        std::process::id()
    ));
    run(
        Command::new("sudo")
            .args(["install", "-m", "0755", "--"])
            .arg(binary)
            .arg(&privileged_staging),
        "stage the verified binary with administrator permission",
    )?;
    if let Err(e) = run(
        Command::new("sudo")
            .args(["mv", "-f", "--"])
            .arg(&privileged_staging)
            .arg(target),
        "replace the installed binary",
    ) {
        let _ = Command::new("sudo")
            .args(["rm", "-f", "--"])
            .arg(&privileged_staging)
            .status();
        return Err(e);
    }
    Ok(())
}

pub fn update() -> Result<(), String> {
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    return Err("Verified automatic updates currently support Linux x86_64 only".to_string());

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        let gh_version = run(
            Command::new("gh").arg("version"),
            "check GitHub CLI version",
        )?;
        validate_gh_version(&String::from_utf8_lossy(&gh_version.stdout))?;
        let tag = latest_tag()?;
        let temp_dir = make_private_temp_dir()?;
        let result = (|| {
            run(
                Command::new("gh")
                    .args([
                        "release",
                        "download",
                        &tag,
                        "--repo",
                        REPOSITORY,
                        "--pattern",
                        ASSET,
                        "--dir",
                    ])
                    .arg(&temp_dir),
                "download the release binary",
            )?;
            let binary = temp_dir.join(ASSET);
            run(
                Command::new("gh")
                    .args(["attestation", "verify"])
                    .arg(&binary)
                    .args(["--repo", REPOSITORY, "--signer-workflow", SIGNER_WORKFLOW]),
                "verify the release provenance",
            )?;
            let target = std::env::current_exe()
                .map_err(|e| format!("Could not locate installed Termos binary: {e}"))?;
            install_verified(&binary, &target)
        })();
        let _ = std::fs::remove_dir_all(&temp_dir);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_gh_version, validate_tag};

    #[test]
    fn release_tag_is_constrained_before_cli_use() {
        assert!(validate_tag("v1.2.3").is_ok());
        assert!(validate_tag("release_2026-10").is_ok());
        assert!(validate_tag("; touch /tmp/injected").is_err());
        assert!(validate_tag("").is_err());
    }

    #[test]
    fn github_cli_version_meets_secure_attestation_minimum() {
        assert!(validate_gh_version("gh version 2.102.0 (2026-01-01)\n").is_ok());
        assert!(validate_gh_version("gh version 2.103.1 (2026-01-01)\n").is_ok());
        assert!(validate_gh_version("gh version 2.101.99 (2026-01-01)\n").is_err());
        assert!(validate_gh_version("gh version invalid\n").is_err());
    }
}
