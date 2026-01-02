use anyhow::{Context, Result};
use std::path::PathBuf;
use which::which;

#[derive(Clone)]
pub struct Insights {
    pub is_root: bool,
    pub arch: String,
    #[allow(dead_code)]
    pub plat: String,
    pub os: String,
    pub is_linux: bool,
    pub is_darwin: bool,
    #[allow(dead_code)]
    pub s_arch: String,
    #[allow(dead_code)]
    pub linux_full_id: String,
    pub pkg_mgr: Option<String>,
    pub has_brew: bool,
    pub has_cargo: bool,
    pub app_abspath: PathBuf,
}

impl Insights {
    pub fn new() -> Result<Self> {
        let is_root = nix::unistd::Uid::effective().is_root();

        let arch_str = match std::env::consts::ARCH {
            "x86_64" => "x86_64".to_string(),
            "aarch64" => "aarch64".to_string(),
            "arm" => "armv7l".to_string(),
            _ => std::env::consts::ARCH.to_string(),
        };

        let plat = std::env::consts::ARCH.to_string();

        let os = std::env::consts::OS.to_string();
        let is_linux = os == "linux";
        let is_darwin = os == "macos";

        let s_arch = if is_darwin {
            if plat == "x86_64" {
                "amd64".to_string()
            } else {
                "arm64".to_string()
            }
        } else {
            match plat.as_str() {
                "aarch64" => "arm64".to_string(),
                "armv7l" => "armv7".to_string(),
                _ => "amd64".to_string(),
            }
        };

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

        let app_abspath =
            std::env::current_exe().with_context(|| "Failed to get executable path")?;

        Ok(Self {
            is_root,
            arch: arch_str,
            plat,
            os,
            is_linux,
            is_darwin,
            s_arch,
            linux_full_id,
            pkg_mgr,
            has_brew,
            has_cargo,
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
