use std::{
    ffi::OsStr,
    process::{Command, Output},
    time::Duration,
};

use clap::Subcommand;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{logger::Logger, notifier::Notifier};

pub struct Microphone;
const PROGRAM: &str = "amixer";
const MUTE_COMMAND: &[&str] = &["-D", "pulse", "set", "Capture", "1+", "toggle"];

impl Microphone {
    pub fn working() -> anyhow::Result<bool> {
        Ok(Self::get_data()?.stderr.is_empty())
    }

    fn get_data() -> anyhow::Result<Output> {
        Self::exec(&["get", "Capture"])
    }

    fn muted() -> anyhow::Result<bool> {
        Self::get_state().map(|v| v.muted)
    }

    // fn get() -> anyhow::Result<u32> {
    //     if Self::working()? {
    //         Self::get_state().map(|v| v.value)
    //     } else {
    //         Ok(0)
    //     }
    // }

    fn set(percent: u32) -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["sset", "Capture", &format!("{percent}%")])?;
        }
        Self::update(0)
    }

    fn up(percent: u32) -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["sset", "Capture", &format!("{percent}%+")])?;
        }
        Self::update(0)
    }

    fn down(percent: u32) -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["sset", "Capture", &format!("{percent}%-")])?;
        }
        Self::update(0)
    }

    fn exec(args: &[impl AsRef<OsStr>]) -> anyhow::Result<Output> {
        let output = Command::new(PROGRAM).args(args).output()?;
        Ok(output)
    }

    fn get_state() -> anyhow::Result<MicrophoneState> {
        if Self::working()? {
            let reg = Regex::new(r"(?m)^.* \[(?<percent>\d+)%\] \[(?<mute>.*)\]$")?;
            let output = String::from_utf8(Self::get_data()?.stdout)?;
            let data = reg
                .captures(&output)
                .ok_or_else(|| anyhow::anyhow!("invalid output command"))?;

            Ok(MicrophoneState::new(
                true,
                data["mute"].trim() != "on",
                data["percent"].trim().parse::<u32>()?,
            ))
        } else {
            Ok(MicrophoneState::new(false, false, 0))
        }
    }

    fn mute() -> anyhow::Result<()> {
        Self::exec(MUTE_COMMAND)?;
        Self::update(0)
    }

    pub fn handle(operation: MicrophoneOp) -> anyhow::Result<()> {
        match operation {
            MicrophoneOp::Mute => Self::mute(),
            MicrophoneOp::Up { percent } => Self::up(percent),
            MicrophoneOp::Down { percent } => Self::down(percent),
            MicrophoneOp::Set { percent } => Self::set(percent),
            MicrophoneOp::Update => Self::update(500),
        }
    }

    fn update(delay: u64) -> anyhow::Result<()> {
        if delay != 0 {
            std::thread::sleep(Duration::from_millis(delay));
        }
        let state = Self::get_state()?;
        state.notify()?;
        state.update_view()
    }

    pub(crate) fn init_view() -> anyhow::Result<()> {
        let state = Self::get_state()?;
        state.update_view()
    }
}
#[derive(Copy, Clone, Subcommand)]
pub enum MicrophoneOp {
    /// Increase by percentage
    Up {
        #[arg(default_value_t = 5, value_parser = clap::value_parser!(u32).range(0..=100))]
        percent: u32,
    },
    /// Decrease by percentage
    Down {
        #[arg(default_value_t = 5, value_parser = clap::value_parser!(u32).range(0..=100))]
        percent: u32,
    },
    /// Set to a percentage
    Set {
        #[arg(value_parser = clap::value_parser!(u32).range(0..=100))]
        percent: u32,
    },
    /// Toggle mute
    Mute,
    /// Update status and notify
    Update,
}

#[derive(Serialize, Deserialize, Debug)]
struct MicrophoneState {
    value: u32,
    working: bool,
    pub muted: bool,
    icon: String,
}

impl MicrophoneState {
    pub fn new(working: bool, muted: bool, value: u32) -> Self {
        let icon = if !working || muted { "󰍭 " } else { "󰍮 " }.to_string();
        Self {
            working,
            muted,
            icon,
            value,
        }
    }

    pub fn notify(&self) -> anyhow::Result<()> {
        let notifier = Notifier::new("microphone");
        if !self.working {
            notifier.send("Microphone", "No output", None, None)
        } else if self.muted {
            notifier.send("Microhone", "Muted", None, None)
        } else {
            notifier.send(
                "Microphone",
                &format!("Set to {}%", self.value),
                None,
                Some(self.value),
            )
        }
    }

    pub fn update_view(&self) -> anyhow::Result<()> {
        Logger::new("microphone-json").send(self)
    }
}
