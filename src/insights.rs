use anyhow::{Context, Result};
use std::path::PathBuf;
use which::which;

#[derive(Clone)]
pub struct Insights {
    pub is_root: bool,
    pub arch: String,
    pub os: String,
    pub is_linux: bool,
    pub is_darwin: bool,
    pub linux_full_id: String,
    pub pkg_mgr: Option<String>,
    pub has_brew: bool,
    pub has_cargo: bool,
    pub has_npm: bool,
    pub has_pip: bool,
    pub has_uv: bool,
    pub has_rustup: bool,
    pub has_flatpak: bool,
    pub has_snap: bool,
    pub has_vscode: bool,
    pub has_docker: bool,
    pub has_gem: bool,
    pub has_deno: bool,
    pub vscode_bin: Option<String>,
    pub has_yarn: bool,
    pub has_go: bool,
    pub app_abspath: PathBuf,
}

impl Insights {
    pub fn new() -> Result<Self> {
        let is_root = nix::unistd::Uid::effective().is_root();

        let arch_str = std::env::consts::ARCH.to_string();

        let os = std::env::consts::OS.to_string();
        let is_linux = os == "linux";
        let is_darwin = os == "macos";

        let mut linux_full_id = String::new();
        let mut pkg_mgr: Option<String> = None;

        if is_linux {
            linux_full_id = Self::get_linux_id()?;

            pkg_mgr = match linux_full_id.as_str() {
                id if id.contains("debian") || id.contains("ubuntu") => Some("apt-get".to_string()),
                id if id.contains("centos")
                    || id.contains("redhat")
                    || id.contains("rhel")
                    || id.contains("rocky")
                    || id.contains("fedora")
                    || id.contains("ol") =>
                {
                    Some("dnf".to_string())
                }
                id if id.contains("alpine") => Some("apk".to_string()),
                id if id.contains("nixos") => Some("nix-env".to_string()),
                id if id.contains("arch") => Some("pacman".to_string()),
                _ => None,
            };
        }

        let has_brew = which("brew").is_ok();
        let has_cargo = which("cargo").is_ok();
        let has_npm = which("npm").is_ok();
        let has_pip = which("pip3").or_else(|_| which("pip")).is_ok();
        let has_uv = which("uv").is_ok();
        let has_rustup = which("rustup").is_ok();
        let has_flatpak = which("flatpak").is_ok();
        let has_snap = which("snap").is_ok();
        let has_vscode = which("code").or_else(|_| which("cursor")).is_ok();
        let has_docker = which("docker").is_ok();
        let has_gem = which("gem").is_ok();
        let has_deno = which("deno").is_ok();

        let vscode_bin = which("code")
            .or_else(|_| which("cursor"))
            .ok()
            .map(|p| p.to_string_lossy().to_string());
        let has_yarn = which("yarn").is_ok();
        let has_go = which("go").is_ok();
        let app_abspath =
            std::env::current_exe().with_context(|| "Failed to get executable path")?;

        Ok(Self {
            is_root,
            arch: arch_str,
            os,
            is_linux,
            is_darwin,
            linux_full_id,
            pkg_mgr,
            has_brew,
            has_cargo,
            has_npm,
            has_pip,
            has_uv,
            has_rustup,
            has_flatpak,
            has_snap,
            has_vscode,
            has_docker,
            has_gem,
            has_deno,
            vscode_bin,
            has_yarn,
            has_go,
            app_abspath,
        })
    }

    fn get_linux_id() -> Result<String> {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg("grep -E -h '^(ID=|ID_LIKE)' /etc/*-release 2>/dev/null | cut -d= -f2- | sed -e 's/\"//g' | xargs")
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok(String::new())
        }
    }
}
