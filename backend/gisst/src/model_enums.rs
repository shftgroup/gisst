use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(rename_all = "lowercase", type_name = "environment_framework")]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    RetroArch,
    V86,
}

impl FromStr for Framework {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "retroarch" => Ok(Framework::RetroArch),
            "v86" => Ok(Framework::V86),
            _ => Err("Attempting to convert Framework that does not exist."),
        }
    }
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Framework::RetroArch => write!(f, "retroarch"),
            Framework::V86 => write!(f, "v86"),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase", type_name = "platform")]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    NES,
    SNES,
    N64,
    DOS,
}

impl FromStr for Platform {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Microsoft Disk Operating System" => Ok(Platform::DOS),
            "Nintendo Entertainment System" => Ok(Platform::NES),
            "Super Nintendo Entertainment System" => Ok(Platform::SNES),
            "Nintendo 64" => Ok(Platform::N64),
            _ => Err("Attempting to convert Platform that does not exist"),
        }
    }
}
