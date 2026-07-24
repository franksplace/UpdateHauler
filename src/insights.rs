use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use which::which;

#[derive(Clone)]
pub struct Insights {
    pub is_root: bool,
    pub arch: String,
    pub os: String,
    pub is_linux: bool,
    pub is_darwin: bool,
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
    pub is_cargo_install: bool,
}

impl Insights {
    pub fn new() -> Result<Self> {
        let is_root = nix::unistd::Uid::effective().is_root();

        let arch_str = std::env::consts::ARCH.to_string();

        let os = std::env::consts::OS.to_string();
        let is_linux = os == "linux";
        let is_darwin = os == "macos";

        let mut pkg_mgr: Option<String> = None;

        if is_linux {
            let linux_full_id = Self::get_linux_id()?;

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

        let cargo_bin_dir = std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .map(|h| h.join(".cargo/bin"));
        let is_cargo_install = cargo_bin_dir
            .as_ref()
            .map_or(false, |d| app_abspath.starts_with(d));

        Ok(Self {
            is_root,
            arch: arch_str,
            os,
            is_linux,
            is_darwin,
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
            is_cargo_install,
        })
    }

    fn get_linux_id() -> Result<String> {
        let mut ids = Vec::new();
        let release_files = ["/etc/os-release", "/etc/lsb-release"];

        for path in &release_files {
            if let Ok(content) = fs::read_to_string(path) {
                for line in content.lines() {
                    if let Some(value) = Self::parse_release_line(line, "ID=")
                        .or_else(|| Self::parse_release_line(line, "ID_LIKE="))
                    {
                        for id in value.split_whitespace() {
                            if !ids.contains(&id.to_string()) {
                                ids.push(id.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(ids.join(" "))
    }

    fn parse_release_line(line: &str, prefix: &str) -> Option<String> {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(prefix) {
            let value = rest.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
        None
    }
}
