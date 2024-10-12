use core::ffi::CStr;

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
    ForceMetic,
    UseSwitchValue,
    ResetAccumulationCounter,
    EnableExternalTbInput,
    DisableExternalTbInput,
}

impl Command {}

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
