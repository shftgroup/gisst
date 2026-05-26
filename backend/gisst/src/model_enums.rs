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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(rename_all = "lowercase", type_name = "platform")]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    NES,
    SNES,
    N64,
    DOS,
    PSX,
    Atari2600,
    AtariST,
    Gameboy,
    GameboyAdvance,
    Windows,
}

impl FromStr for Platform {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Microsoft Disk Operating System" => Ok(Platform::DOS),
            "Microsoft Windows" => Ok(Platform::Windows),
            "Nintendo Entertainment System" => Ok(Platform::NES),
            "Super Nintendo Entertainment System" => Ok(Platform::SNES),
            "Nintendo 64" => Ok(Platform::N64),
            "Sony PlayStation" => Ok(Platform::PSX),
            "Atari 2600" => Ok(Platform::Atari2600),
            "Atari ST" => Ok(Platform::AtariST),
            "Nintendo Game Boy" => Ok(Platform::Gameboy),
            "Nintendo Game Boy Advance" => Ok(Platform::GameboyAdvance),

            _ => Err("Attempting to convert Platform that does not exist"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framework_parses_retroarch() {
        assert_eq!(
            "retroarch".parse::<Framework>().unwrap(),
            Framework::RetroArch
        );
    }

    #[test]
    fn framework_parses_v86() {
        assert_eq!("v86".parse::<Framework>().unwrap(), Framework::V86);
    }

    #[test]
    fn framework_rejects_unknown_string() {
        assert!("mame".parse::<Framework>().is_err());
    }

    #[test]
    fn platform_parses_all_known_strings() {
        let cases = [
            ("Microsoft Disk Operating System", Platform::DOS),
            ("Microsoft Windows", Platform::Windows),
            ("Nintendo Entertainment System", Platform::NES),
            ("Super Nintendo Entertainment System", Platform::SNES),
            ("Nintendo 64", Platform::N64),
            ("Sony PlayStation", Platform::PSX),
            ("Atari 2600", Platform::Atari2600),
            ("Atari ST", Platform::AtariST),
            ("Nintendo Game Boy", Platform::Gameboy),
            ("Nintendo Game Boy Advance", Platform::GameboyAdvance),
        ];
        for (input, expected) in cases {
            let result = input.parse::<Platform>();
            assert!(result.is_ok(), "'{input}' should parse okay");
            assert_eq!(expected, result.unwrap());
        }
    }

    #[test]
    fn platform_error_unknown_string() {
        assert!("Fantasy Console".parse::<Platform>().is_err());
    }

    #[test]
    fn platform_error_empty_string() {
        assert!("".parse::<Platform>().is_err());
    }
}
