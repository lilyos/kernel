use std::fmt::Display;

use anyhow::{bail, Result};
use duct::cmd;

#[derive(Clone, Copy, Debug)]
pub struct Version(pub u64, pub u64, pub u64);

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.0, self.1, self.2))
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ToolInfo {
    pub present: bool,
    pub version: Option<Version>,
}

#[derive(Clone, Copy, Debug)]
pub enum Tool {
    Git(ToolInfo),
    QemuX86_64(ToolInfo),
}

impl ToString for Tool {
    fn to_string(&self) -> String {
        match self {
            Tool::Git(_) => "git".to_owned(),
            Tool::QemuX86_64(_) => "qemu".to_owned(),
        }
    }
}

impl TryFrom<&str> for Tool {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "git" => Tool::Git(Default::default()),
            "qemu" => Tool::QemuX86_64(Default::default()),
            _ => bail!("Unknown tool"),
        })
    }
}

pub struct ToolDescriptor<'a> {
    pub name: &'a str,
    pub info: ToolInfo,
}

impl From<Tool> for ToolDescriptor<'_> {
    fn from(tool: Tool) -> Self {
        match tool {
            Tool::Git(info) => ToolDescriptor { name: "Git", info },
            Tool::QemuX86_64(info) => ToolDescriptor {
                name: "Qemu (x86_64)",
                info,
            },
        }
    }
}

impl From<&Tool> for ToolDescriptor<'_> {
    fn from(tool: &Tool) -> Self {
        From::<Tool>::from(*tool)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PackageManager {
    AptGet,
    Dnf,
    Pacman,
    Chocolatey,
    Scoop,
}

lazy_static::lazy_static! {
    static ref SEMVER_REGEX: Regex = {
        Regex::new(r#"\d*\.\d*\.\d*"#).unwrap()
    };

    static ref PACKAGE_MANAGER: PackageManager = {
        if cmd!("dnf", "--version").stdout_null().run().is_ok() {
            PackageManager::Dnf
        } else if cmd!("apt-get", "--version").stdout_null().run().is_ok() {
            PackageManager::AptGet
        } else if cmd!("pacman", "--version").stdout_null().run().is_ok() {
            PackageManager::Pacman
        } else if cmd!("choco", "--version").stdout_null().run().is_ok() {
            PackageManager::Chocolatey
        } else if cmd!("scoop", "--version").stdout_null().run().is_ok() {
            PackageManager::Scoop
        } else {
            panic!("wtf are you running?")
        }
    };
}

pub fn get_semver_from_str(text: &str) -> Option<Version> {
    let caps = SEMVER_REGEX.captures(text)?;
    let nums = caps
        .get(0)?
        .as_str()
        .split('.')
        .flat_map(|i| i.parse().ok())
        .collect::<Vec<u64>>();

    if nums.len() != 3 {
        return None;
    }

    Some(Version(nums[0], nums[1], nums[2]))
}

use log::info;
use regex::Regex;

pub fn get_tools() -> Vec<Tool> {
    let mut tools = Vec::new();

    if let Ok(vs) = cmd!("git", "--version").read() {
        tools.push(Tool::Git(ToolInfo {
            present: true,
            version: get_semver_from_str(&vs),
        }));
    } else {
        tools.push(Tool::QemuX86_64(ToolInfo::default()));
    }

    if let Ok(vs) = cmd!("qemu-system-x86_64", "--version").read() {
        tools.push(Tool::QemuX86_64(ToolInfo {
            present: true,
            version: get_semver_from_str(&vs),
        }));
    } else {
        tools.push(Tool::QemuX86_64(ToolInfo::default()))
    }

    tools
}

pub fn print_tools(tools: &[Tool]) {
    const YES: char = '✓';
    const NO: char = '✗';

    for tool in tools {
        let desc: ToolDescriptor = tool.into();
        info!(
            "{} {} (Version {})",
            if desc.info.present { YES } else { NO },
            desc.name,
            if let Some(version) = desc.info.version {
                version.to_string()
            } else {
                "???".to_owned()
            }
        )
    }
}

pub fn install_tools(tools: &[String]) -> Result<()> {
    let tools = tools
        .iter()
        .flat_map(|i| i.as_str().try_into().ok())
        .collect::<Vec<Tool>>();
    for tool in tools.iter() {
        invoke_installer_for(*tool)?;
    }
    Ok(())
}

pub fn invoke_installer_for(tool: Tool) -> Result<()> {
    let tool = tool.to_string();
    match *PACKAGE_MANAGER {
        PackageManager::AptGet => {
            cmd!("apt-get", "update").run()?;
            cmd!("apt-get", "install", tool, "-y").run()?;
        }
        PackageManager::Dnf => {
            cmd!("dnf", "install", tool, "-y").run()?;
        }
        PackageManager::Pacman => {
            cmd!("pacman", "-Syy").run()?;
            cmd!("pacman", "-S", tool).run()?;
        }
        PackageManager::Chocolatey => {
            cmd!("choco", "install", tool).run()?;
        }
        PackageManager::Scoop => {
            cmd!("scoop", "install", tool).run()?;
        }
    }
    Ok(())
}

pub fn uninstall_tools(tools: &[String]) -> Result<()> {
    let tools = tools
        .iter()
        .flat_map(|i| i.as_str().try_into().ok())
        .collect::<Vec<Tool>>();
    for tool in tools.iter() {
        invoke_uninstaller_for(*tool)?;
    }
    Ok(())
}

pub fn invoke_uninstaller_for(tool: Tool) -> Result<()> {
    let tool = tool.to_string();
    match *PACKAGE_MANAGER {
        PackageManager::AptGet => {
            cmd!("apt-get", "remove", tool).run()?;
        }
        PackageManager::Dnf => {
            cmd!("dnf", "remove", tool).run()?;
        }
        PackageManager::Pacman => {
            cmd!("pacman", "-R", tool).run()?;
        }
        PackageManager::Chocolatey => {
            cmd!("choco", "uninstall", tool).run()?;
        }
        PackageManager::Scoop => {
            cmd!("scoop", "uninstall", tool).run()?;
        }
    }
    Ok(())
}
