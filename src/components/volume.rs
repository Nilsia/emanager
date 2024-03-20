use crate::notifier::Notifier;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::process::{Command, Output};

use super::components::{NotifiableState, ScaledComponent};

const PROGRAM: &str = "wpctl";
const ID: &str = "@DEFAULT_AUDIO_SINK@";
const JSON_VIEW_NAME: &str = "volume-json";

pub struct Volume;

impl Volume {
    pub fn working() -> anyhow::Result<bool> {
        Ok(Self::exec(&["get-volume", ID])?.stderr.is_empty())
    }

    pub fn muted() -> anyhow::Result<bool> {
        let string = String::from_utf8(Self::exec(&["get-volume", ID])?.stdout)?;
        Ok(string.contains("MUTED"))
    }

    pub fn mute() -> anyhow::Result<()> {
        Self::exec(&["set-mute", ID, "toggle"])?;
        Self::update(0)
    }

    pub fn handle(operation: VolumeOp) -> anyhow::Result<()> {
        match operation {
            VolumeOp::Up { percent } => Self::up(percent),
            VolumeOp::Down { percent } => Self::down(percent),
            VolumeOp::Set { percent } => Self::set(percent),
            VolumeOp::Mute => Self::mute(),
            VolumeOp::Update => Self::update(500),
        }
    }

    fn exec(args: &[impl AsRef<OsStr>]) -> anyhow::Result<Output> {
        let output = Command::new(PROGRAM).args(args).output()?;
        Ok(output)
    }

    pub(crate) fn init_view() -> anyhow::Result<()> {
        let (working, muted, value) = (Self::working()?, Self::muted()?, Self::get()?);
        let state = VolumeState::new(value, muted, working);
        state.update_view()
    }
}

impl ScaledComponent<VolumeState> for Volume {
    fn get() -> anyhow::Result<u32> {
        if Self::working()? {
            let string = String::from_utf8(Self::exec(&["get-volume", ID])?.stdout)?;
            let volume = string.split(' ').collect::<Vec<&str>>()[1]
                .trim()
                .parse::<f32>()?;
            Ok((volume * 100.).round() as u32)
        } else {
            Ok(0)
        }
    }

    fn set(percent: u32) -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["set-volume", ID, &format!("{percent}%"), "-l", "1"])?;
        }
        Self::update(0)
    }

    fn up(percent: u32) -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["set-volume", ID, &format!("{percent}%+"), "-l", "1"])?;
        }
        Self::update(0)
    }

    fn down(percent: u32) -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["set-volume", ID, &format!("{percent}%-")])?;
        }
        Self::update(0)
    }

    fn get_state() -> anyhow::Result<VolumeState> {
        Ok(VolumeState::new(
            Self::get()?,
            Self::muted()?,
            Self::working()?,
        ))
    }
}

#[derive(Copy, Clone, Subcommand)]
pub enum VolumeOp {
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

#[derive(Serialize, Deserialize)]
struct VolumeState {
    value: u32,
    muted: bool,
    working: bool,
    icon: String,
}

impl VolumeState {
    pub fn new(value: u32, muted: bool, working: bool) -> Self {
        let icon = if !working || muted {
            "󰝟 "
        } else if value >= 40 {
            "󰕾 "
        } else if value >= 20 {
            "󰖀 "
        } else {
            "󰕿 "
        }
        .to_string();
        Self {
            value,
            muted,
            working,
            icon,
        }
    }
}

impl NotifiableState for VolumeState {
    fn notify(&self) -> anyhow::Result<()> {
        let notifier = Notifier::new("volume");
        if !self.working {
            notifier.send("Volume", "No output", None, None)
        } else if self.muted {
            notifier.send("Volume", "Muted", None, None)
        } else {
            notifier.send(
                "Volume",
                &format!("Set to {}%", self.value),
                None,
                Some(self.value),
            )
        }
    }

    fn json_name(&self) -> &str {
        JSON_VIEW_NAME
    }
}
