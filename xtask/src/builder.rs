use std::str::FromStr;

use anyhow::Result;

use thiserror::Error;

#[derive(Clone, Copy, Debug, Error)]
pub enum TargetError {
    #[error("Unknown architecture")]
    UnknownArchitecture,
    #[error("Unknown firwmare")]
    UnknownFirmware,
    #[error("Unknown bootloader")]
    UnknownBootloader,
    #[error("The target triple has an improper amount of fields")]
    ImproperFieldCount,
}

#[derive(Clone, Copy, Debug)]
pub enum TargetArch {
    X86_64,
}

impl FromStr for TargetArch {
    type Err = TargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "x86_64" => Self::X86_64,
            _ => return Err(TargetError::UnknownArchitecture),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TargetFirmware {
    Bios,
    Uefi,
    Sbi,
    TrustedFirmwareA,
}

impl FromStr for TargetFirmware {
    type Err = TargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "bios" => Self::Bios,
            "uefi" => Self::Uefi,
            "sbi" => Self::Sbi,
            "trustedfirmwarea" | "tfa" => Self::TrustedFirmwareA,
            _ => return Err(TargetError::UnknownFirmware),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TargetBootloader {
    Limine,
}

impl FromStr for TargetBootloader {
    type Err = TargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "limine" => Self::Limine,
            _ => return Err(TargetError::UnknownBootloader),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Target {
    arch: TargetArch,
    firmware: TargetFirmware,
    bootloader: TargetBootloader,
}

impl FromStr for Target {
    type Err = TargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('-').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(TargetError::ImproperFieldCount);
        }

        let arch = parts[0].parse()?;
        let firmware = parts[1].parse()?;
        let bootloader = parts[2].parse()?;

        Ok(Self {
            arch,
            firmware,
            bootloader,
        })
    }
}

pub fn build_target(release: bool, target: Target) -> Result<()> {
    anyhow::bail!("Unimplemented")
}
