use core::ffi::CStr;
use core::fmt::Write;

use alloc::{ffi::CString, string::String};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    AccumulationData,
    ReadAvailableData,
    Kill,
    BaudRate(BaudRate),
    PollingMode,
    ContinousMode,
    ForceHighResolution,
    ForceLowResolution,
    ForceImperial,
    ForceMetric,
    UseSwitchValue,
    ResetAccumulationCounter,
    EnableExternalTbInput,
    DisableExternalTbInput,
}

impl Command {
    pub fn list() -> &'static [Command] {
        use Command as C;
        &[
            C::AccumulationData,
            C::ReadAvailableData,
            C::Kill,
            // C::BaudRate(_), skip for now
            C::PollingMode,
            C::ContinousMode,
            C::ForceHighResolution,
            C::ForceLowResolution,
            C::ForceImperial,
            C::ForceMetric,
            C::UseSwitchValue,
            C::ResetAccumulationCounter,
            C::EnableExternalTbInput,
            C::DisableExternalTbInput,
        ]
    }

    pub fn name(self) -> &'static CStr {
        match self {
            Command::AccumulationData => c"Get Accumulation Data",
            Command::ReadAvailableData => c"Read Available Data",
            Command::Kill => c"Kill",
            Command::BaudRate(_) => c"Set Baud Rate",
            Command::PollingMode => c"Set Polling Mode",
            Command::ContinousMode => c"Set Continous Mode",
            Command::ForceHighResolution => c"Force High Resolution",
            Command::ForceLowResolution => c"Force Low Resolution",
            Command::ForceImperial => c"Force Imperial Units",
            Command::ForceMetric => c"Force Metric Units",
            Command::UseSwitchValue => c"Use Values from Switch",
            Command::ResetAccumulationCounter => c"Reset Acc Counter",
            Command::EnableExternalTbInput => c"Enable External TB Input",
            Command::DisableExternalTbInput => c"Disable External TB Input",
        }
    }

    pub fn code(&self) -> u32 {
        match self {
            Command::AccumulationData => 0,
            Command::ReadAvailableData => 1,
            Command::Kill => 2,
            Command::BaudRate(BaudRate::Baud1200) => 1200 + 3,
            Command::BaudRate(BaudRate::Baud2400) => 2400 + 3,
            Command::BaudRate(BaudRate::Baud4800) => 4800 + 3,
            Command::BaudRate(BaudRate::Baud9600) => 9600 + 3,
            Command::BaudRate(BaudRate::Baud19200) => 19200 + 3,
            Command::BaudRate(BaudRate::Baud38400) => 38400 + 3,
            Command::BaudRate(BaudRate::Baud57600) => 57600 + 3,
            Command::PollingMode => 4,
            Command::ContinousMode => 5,
            Command::ForceHighResolution => 6,
            Command::ForceLowResolution => 7,
            Command::ForceImperial => 8,
            Command::ForceMetric => 9,
            Command::UseSwitchValue => 10,
            Command::ResetAccumulationCounter => 11,
            Command::EnableExternalTbInput => 12,
            Command::DisableExternalTbInput => 13,
        }
    }

    pub fn try_from_code(code: u32) -> Option<Command> {
        Some(match code {
            0 => Command::AccumulationData,
            1 => Command::ReadAvailableData,
            2 => Command::Kill,
            1203 => Command::BaudRate(BaudRate::Baud1200),
            2403 => Command::BaudRate(BaudRate::Baud2400),
            4803 => Command::BaudRate(BaudRate::Baud4800),
            9603 => Command::BaudRate(BaudRate::Baud9600),
            19203 => Command::BaudRate(BaudRate::Baud19200),
            38403 => Command::BaudRate(BaudRate::Baud38400),
            57603 => Command::BaudRate(BaudRate::Baud57600),
            4 => Command::PollingMode,
            5 => Command::ContinousMode,
            6 => Command::ForceHighResolution,
            7 => Command::ForceLowResolution,
            8 => Command::ForceImperial,
            9 => Command::ForceMetric,
            10 => Command::UseSwitchValue,
            11 => Command::ResetAccumulationCounter,
            12 => Command::EnableExternalTbInput,
            13 => Command::DisableExternalTbInput,
            _ => return None,
        })
    }

    pub fn cmd(&self) -> CString {
        let mut s = String::new();
        match self {
            Command::AccumulationData => s.push('a'),
            Command::ReadAvailableData => s.push('r'),
            Command::Kill => s.push('k'),
            Command::BaudRate(b) => write!(s, "b{}", b.code()).expect("infallible"),
            Command::PollingMode => s.push('p'),
            Command::ContinousMode => s.push('c'),
            Command::ForceHighResolution => s.push('h'),
            Command::ForceLowResolution => s.push('l'),
            Command::ForceImperial => s.push('i'),
            Command::ForceMetric => s.push('m'),
            Command::UseSwitchValue => s.push('s'),
            Command::ResetAccumulationCounter => s.push('o'),
            Command::EnableExternalTbInput => s.push('x'),
            Command::DisableExternalTbInput => s.push('y'),
        }
        s.push_str("\r\n");

        CString::new(s).expect("all possible options should create a valid c-string")
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum BaudRate {
    Baud1200 = 0,
    Baud2400 = 1,
    Baud4800 = 2,
    Baud9600 = 3, // Default
    Baud19200 = 4,
    Baud38400 = 5,
    Baud57600 = 6,
}

impl Default for BaudRate {
    fn default() -> Self {
        BaudRate::Baud9600
    }
}

impl BaudRate {
    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn rate(self) -> u16 {
        match self {
            BaudRate::Baud1200 => 1200,
            BaudRate::Baud2400 => 2400,
            BaudRate::Baud4800 => 4800,
            BaudRate::Baud9600 => 9600,
            BaudRate::Baud19200 => 19200,
            BaudRate::Baud38400 => 38400,
            BaudRate::Baud57600 => 57600,
        }
    }

    pub fn rate_as_char(self) -> &'static CStr {
        match self {
            BaudRate::Baud1200 => c"1200",
            BaudRate::Baud2400 => c"2400",
            BaudRate::Baud4800 => c"4800",
            BaudRate::Baud9600 => c"9600",
            BaudRate::Baud19200 => c"19200",
            BaudRate::Baud38400 => c"38400",
            BaudRate::Baud57600 => c"57600",
        }
    }

    pub fn list() -> [Self; 7] {
        [
            BaudRate::Baud1200,
            BaudRate::Baud2400,
            BaudRate::Baud4800,
            BaudRate::Baud9600,
            BaudRate::Baud19200,
            BaudRate::Baud38400,
            BaudRate::Baud57600,
        ]
    }
}

impl TryFrom<u16> for BaudRate {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use BaudRate as BR;
        Ok(match value {
            0 | 1200 => BR::Baud1200,
            1 | 2400 => BR::Baud2400,
            2 | 4800 => BR::Baud4800,
            3 | 9600 => BR::Baud9600,
            4 | 19200 => BR::Baud19200,
            5 | 38400 => BR::Baud38400,
            6 | 57600 => BR::Baud57600,
            _ => return Err(value),
        })
    }
}
