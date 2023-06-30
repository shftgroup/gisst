use serde::{Deserialize, Serialize};
use std::{str::FromStr};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase", type_name = "object_role")]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase", type_name = "object_role")]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    NES,
    SNES,
    DOS,
}

impl FromStr for Platform {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Microsoft Disk Operating System" => Ok(Platform::DOS),
            "Nintendo Entertainment System" => Ok(Platform::NES),
            "Super Nintendo Entertainment System" => Ok(Platform::SNES),
            _ => Err("Attempting to convert Platform that does not exist"),
        }
    }
}
