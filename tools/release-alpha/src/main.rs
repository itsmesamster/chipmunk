use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const ALPHA_APP_NAME: &str = "chipmunk-alpha.app";
const ALPHA_BUNDLE_ID: &str = "com.esrlabs.chipmunk.alpha";

struct Cli {
    code_sign: Option<PathBuf>,
}

struct AlphaMacOsCodeSign {
    env_vars: AlphaEnvVars,
    notarize_command: AlphaNotarizeCommand,
}

struct AlphaEnvVars {
    check_enabled: Vec<String>,
    check_disabled: Vec<String>,
    signing_id: String,
}

struct AlphaNotarizeCommand {
    command: String,
    env_apple_id: String,
    env_team_id: String,
    env_password: String,
    accepted_line: String,
}

fn main() -> Result<(), String> {
    let cli = parse_args()?;
    let code_sign = match cli.code_sign {
        Some(path) => Some(AlphaMacOsCodeSign::load(path)?),
        None => None,
    };

    clean_release()?;
    build_alpha()?;

    let version = alpha_version()?;
    let archive = if cfg!(target_os = "macos") {
        package_macos(&version, code_sign.as_ref())?
    } else {
        package_portable(&version)?
    };

    println!("Alpha release artifact created: {}", archive.display());
    Ok(())
}

fn parse_args() -> Result<Cli, String> {
    let mut args = std::env::args().skip(1);
    let mut code_sign = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("Usage: release-alpha [--code-sign <PATH>]");
                std::process::exit(0);
            }
            "-c" | "--code-sign" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --code-sign"))?;
                code_sign = Some(PathBuf::from(value));
            }
            unknown => return Err(format!("Unknown argument: {unknown}")),
        }
    }

    Ok(Cli { code_sign })
}

fn build_alpha() -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["build", "--release", "--locked", "--manifest-path", "Cargo.toml"])
        .current_dir(alpha_app_root())
        .status()
        .map_err(|err| format!("Building Chipmunk Alpha failed: {err}"))?;

    ensure(status.success(), "Building Chipmunk Alpha failed")?;
    Ok(())
}

fn package_portable(version: &str) -> Result<PathBuf, String> {
    let archive_root = format!("chipmunk-alpha@{version}-{}-portable", platform_name());
    let staging_dir = alpha_release_path().join(&archive_root);

    fs::create_dir_all(&staging_dir).map_err(|err| {
        format!(
            "Creating alpha staging directory failed ({}): {err}",
            staging_dir.display()
        )
    })?;
    fs::copy(alpha_binary_path()?, staging_dir.join(alpha_binary_name()))
        .map_err(|err| format!("Copying alpha binary failed: {err}"))?;
    fs::copy(repo_readme_path(), staging_dir.join("README.md"))
        .map_err(|err| format!("Copying README for alpha package failed: {err}"))?;

    let archive = if cfg!(target_os = "windows") {
        let archive = alpha_release_path().join(format!("{archive_root}.zip"));
        let status = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Compress-Archive -Path '{}' -DestinationPath '{}'",
                    staging_dir.join("*").display(),
                    archive.display()
                ),
            ])
            .status()
            .map_err(|err| format!("Compressing alpha zip failed: {err}"))?;
        ensure(status.success(), "Compressing alpha zip failed")?;
        archive
    } else {
        let archive = alpha_release_path().join(format!("{archive_root}.tgz"));
        let status = Command::new("tar")
            .args([
                "-czf",
                &archive.to_string_lossy(),
                "-C",
                &alpha_release_path().to_string_lossy(),
                &archive_root,
            ])
            .status()
            .map_err(|err| format!("Compressing alpha archive failed: {err}"))?;
        ensure(status.success(), "Compressing alpha archive failed")?;
        archive
    };

    Ok(archive)
}

fn package_macos(version: &str, code_sign: Option<&AlphaMacOsCodeSign>) -> Result<PathBuf, String> {
    let app_root = alpha_release_path().join(ALPHA_APP_NAME);
    let contents = app_root.join("Contents");
    let macos_dir = contents.join("MacOS");
    let resources_dir = contents.join("Resources");

    fs::create_dir_all(&macos_dir).map_err(|err| {
        format!(
            "Creating app bundle directory failed ({}): {err}",
            macos_dir.display()
        )
    })?;
    fs::create_dir_all(&resources_dir).map_err(|err| {
        format!(
            "Creating app bundle resources directory failed ({}): {err}",
            resources_dir.display()
        )
    })?;

    fs::copy(alpha_binary_path()?, macos_dir.join("chipmunk"))
        .map_err(|err| format!("Copying alpha binary into .app failed: {err}"))?;
    fs::copy(icon_path(), resources_dir.join("icon.icns"))
        .map_err(|err| format!("Copying alpha macOS icon failed: {err}"))?;
    fs::copy(repo_readme_path(), alpha_release_path().join("README.md"))
        .map_err(|err| format!("Copying README for alpha package failed: {err}"))?;

    fs::write(contents.join("Info.plist"), info_plist(version))
        .map_err(|err| format!("Writing Info.plist for alpha app failed: {err}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let binary = macos_dir.join("chipmunk");
        let mut perms = fs::metadata(&binary)
            .map_err(|err| format!("Reading alpha binary metadata failed: {err}"))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(binary, perms)
            .map_err(|err| format!("Setting alpha binary permissions failed: {err}"))?;
    }

    if let Some(code_sign) = code_sign {
        if code_sign.allowed() {
            code_sign.sign_app(&app_root)?;
        } else {
            eprintln!("Skipping code signing because required environment variables are missing.");
        }
    }

    let archive = alpha_release_path().join(format!(
        "chipmunk-alpha@{version}-{}-portable.zip",
        platform_name()
    ));
    zip_macos_bundle(&app_root, &archive)?;

    if let Some(code_sign) = code_sign {
        if code_sign.allowed() {
            code_sign.notarize_archive(&archive)?;
            let status = Command::new("xcrun")
                .args(["stapler", "staple", &app_root.to_string_lossy()])
                .status()
                .map_err(|err| format!("Stapling alpha app failed: {err}"))?;
            ensure(status.success(), "Stapling alpha app failed")?;
            zip_macos_bundle(&app_root, &archive)?;
        }
    }

    Ok(archive)
}

fn zip_macos_bundle(app_root: &Path, archive: &Path) -> Result<(), String> {
    let status = Command::new("ditto")
        .args([
            "-c",
            "-k",
            "--keepParent",
            &app_root.to_string_lossy(),
            &archive.to_string_lossy(),
        ])
        .status()
        .map_err(|err| format!("Creating macOS alpha zip archive failed: {err}"))?;
    ensure(status.success(), "Creating macOS alpha zip archive failed")?;
    Ok(())
}

fn clean_release() -> Result<(), String> {
    let release_path = alpha_release_path();
    if release_path.exists() {
        fs::remove_dir_all(&release_path).map_err(|err| {
            format!(
                "Removing previous alpha release directory failed ({}): {err}",
                release_path.display()
            )
        })?;
    }
    fs::create_dir_all(&release_path).map_err(|err| {
        format!(
            "Creating alpha release directory failed ({}): {err}",
            release_path.display()
        )
    })?;
    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("release-alpha tool must live under <repo>/tools/release-alpha")
        .to_path_buf()
}

fn alpha_app_root() -> PathBuf {
    repo_root()
        .join("application")
        .join("apps")
        .join("indexer")
        .join("gui")
        .join("application")
}

fn alpha_release_path() -> PathBuf {
    alpha_app_root().join("release")
}

fn alpha_workspace_root() -> PathBuf {
    repo_root()
        .join("application")
        .join("apps")
        .join("indexer")
}

fn alpha_binary_path() -> Result<PathBuf, String> {
    let path = alpha_workspace_root()
        .join("target")
        .join("release")
        .join(alpha_binary_name());
    ensure(
        path.exists(),
        &format!("Alpha binary doesn't exist: {}", path.display()),
    )?;
    Ok(path)
}

fn alpha_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "chipmunk.exe"
    } else {
        "chipmunk"
    }
}

fn repo_readme_path() -> PathBuf {
    repo_root().join("README.md")
}

fn icon_path() -> PathBuf {
    repo_root()
        .join("application")
        .join("holder")
        .join("resources")
        .join("mac")
        .join("chipmunk.icns")
}

fn entitlements_path() -> PathBuf {
    repo_root()
        .join("application")
        .join("holder")
        .join("resources")
        .join("mac")
        .join("entitlements.mac.plist")
}

fn alpha_version() -> Result<String, String> {
    let workspace_manifest = alpha_workspace_root().join("Cargo.toml");
    let content = fs::read_to_string(&workspace_manifest).map_err(|err| {
        format!(
            "Reading alpha workspace Cargo manifest failed ({}): {err}",
            workspace_manifest.display()
        )
    })?;

    let section = parse_toml_section(&content, "workspace.package");
    section
        .get("version")
        .cloned()
        .ok_or_else(|| String::from("Resolving alpha version from workspace.package.version failed"))
}

fn platform_name() -> String {
    let mut platform = if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "macos") {
        "darwin".to_string()
    } else if cfg!(target_os = "windows") {
        "win64".to_string()
    } else {
        panic!(
            "Unknown target os: {}, arch: {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    };

    if cfg!(target_arch = "aarch64") {
        platform.push_str("-arm64");
    }

    platform
}

fn info_plist(version: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>Chipmunk Alpha</string>
  <key>CFBundleExecutable</key>
  <string>chipmunk</string>
  <key>CFBundleIconFile</key>
  <string>icon.icns</string>
  <key>CFBundleIdentifier</key>
  <string>{ALPHA_BUNDLE_ID}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Chipmunk Alpha</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>{version}</string>
  <key>CFBundleVersion</key>
  <string>{version}</string>
</dict>
</plist>
"#
    )
}

impl AlphaMacOsCodeSign {
    fn load(config_path: PathBuf) -> Result<Self, String> {
        ensure(
            cfg!(target_os = "macos"),
            "Alpha code signing is only supported on macOS",
        )?;
        let content = fs::read_to_string(&config_path).map_err(|err| {
            format!(
                "Reading alpha code signing config failed ({}): {err}",
                config_path.display()
            )
        })?;

        let env_vars = parse_toml_section(&content, "env_vars");
        let notarize = parse_toml_section(&content, "notarize_command");

        Ok(Self {
            env_vars: AlphaEnvVars {
                check_enabled: parse_array(
                    env_vars
                        .get("check_enabled")
                        .ok_or_else(|| String::from("Missing env_vars.check_enabled"))?,
                ),
                check_disabled: parse_array(
                    env_vars
                        .get("check_disabled")
                        .ok_or_else(|| String::from("Missing env_vars.check_disabled"))?,
                ),
                signing_id: env_vars
                    .get("signing_id")
                    .cloned()
                    .ok_or_else(|| String::from("Missing env_vars.signing_id"))?,
            },
            notarize_command: AlphaNotarizeCommand {
                command: notarize
                    .get("command")
                    .cloned()
                    .ok_or_else(|| String::from("Missing notarize_command.command"))?,
                env_apple_id: notarize
                    .get("env_apple_id")
                    .cloned()
                    .ok_or_else(|| String::from("Missing notarize_command.env_apple_id"))?,
                env_team_id: notarize
                    .get("env_team_id")
                    .cloned()
                    .ok_or_else(|| String::from("Missing notarize_command.env_team_id"))?,
                env_password: notarize
                    .get("env_password")
                    .cloned()
                    .ok_or_else(|| String::from("Missing notarize_command.env_password"))?,
                accepted_line: notarize
                    .get("accepted_line")
                    .cloned()
                    .ok_or_else(|| String::from("Missing notarize_command.accepted_line"))?,
            },
        })
    }

    fn allowed(&self) -> bool {
        self.env_vars
            .check_enabled
            .iter()
            .all(|name| std::env::var(name).is_ok())
            && self
                .env_vars
                .check_disabled
                .iter()
                .all(|name| std::env::var(name).is_err())
    }

    fn sign_app(&self, app_root: &Path) -> Result<(), String> {
        let signing_id = std::env::var(&self.env_vars.signing_id).map_err(|_| {
            format!(
                "Reading signing identity failed from env var {}",
                self.env_vars.signing_id
            )
        })?;

        let executable = app_root.join("Contents").join("MacOS").join("chipmunk");
        let status = Command::new("codesign")
            .args([
                "--force",
                "--sign",
                &signing_id,
                "--timestamp",
                "--options",
                "runtime",
                "--entitlements",
                &entitlements_path().to_string_lossy(),
                &executable.to_string_lossy(),
            ])
            .status()
            .map_err(|err| format!("Signing alpha executable failed: {err}"))?;
        ensure(status.success(), "Signing alpha executable failed")?;

        let status = Command::new("codesign")
            .args([
                "--force",
                "--sign",
                &signing_id,
                "--timestamp",
                "--options",
                "runtime",
                "--deep",
                "--strict",
                "--entitlements",
                &entitlements_path().to_string_lossy(),
                &app_root.to_string_lossy(),
            ])
            .status()
            .map_err(|err| format!("Signing alpha app bundle failed: {err}"))?;
        ensure(status.success(), "Signing alpha app bundle failed")?;

        let status = Command::new("codesign")
            .args(["--verify", "--verbose=4", &app_root.to_string_lossy()])
            .status()
            .map_err(|err| format!("Verifying alpha app bundle signature failed: {err}"))?;
        ensure(status.success(), "Verifying alpha app bundle signature failed")?;

        Ok(())
    }

    fn notarize_archive(&self, archive: &Path) -> Result<(), String> {
        let apple_id = std::env::var(&self.notarize_command.env_apple_id)
            .map_err(|_| format!("Missing env var {}", self.notarize_command.env_apple_id))?;
        let team_id = std::env::var(&self.notarize_command.env_team_id)
            .map_err(|_| format!("Missing env var {}", self.notarize_command.env_team_id))?;
        let password = std::env::var(&self.notarize_command.env_password)
            .map_err(|_| format!("Missing env var {}", self.notarize_command.env_password))?;

        let mut parts = self.notarize_command.command.split_whitespace();
        let cmd = parts
            .next()
            .ok_or_else(|| String::from("Alpha notarize command must include a binary"))?;
        let mut command = Command::new(cmd);
        command.args(parts);
        command
            .arg(archive)
            .arg("--apple-id")
            .arg(&apple_id)
            .arg("--team-id")
            .arg(&team_id)
            .arg("--password")
            .arg(&password);

        let output = command
            .output()
            .map_err(|err| format!("Running alpha notarize command failed: {err}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let accepted = stdout
            .lines()
            .chain(stderr.lines())
            .any(|line| line.trim() == self.notarize_command.accepted_line);

        ensure(output.status.success(), "Alpha notarize command failed")?;
        ensure(
            accepted,
            "Alpha notarize command did not report accepted status",
        )?;

        Ok(())
    }
}

fn ensure(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

fn parse_toml_section(content: &str, section: &str) -> HashMap<String, String> {
    let mut current = String::new();
    let mut values = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current = line
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
            continue;
        }
        if current != section {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            values.insert(key.trim().to_string(), trim_toml_string(value.trim()));
        }
    }

    values
}

fn trim_toml_string(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_array(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(trim_toml_string)
        .collect()
}
