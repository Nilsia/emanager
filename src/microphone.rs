use std::{
    ffi::OsStr,
    process::{Command, Output},
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub struct Microphone;
const PROGRAM: &str = "amixer";
const MUTE_COMMAND: &[&str] = &["-D", "pulse", "set", "Capture", "1+", "toggle"];

impl Microphone {
    pub fn working() -> anyhow::Result<bool> {
        Ok(Self::exec(&["get-volume"])?.stderr.is_empty())
    }
    fn exec(args: &[impl AsRef<OsStr>]) -> anyhow::Result<Output> {
        let output = Command::new(PROGRAM).args(args).output()?;
        Ok(output)
    }

    pub fn handle(operation: MicrophoneOp) -> anyhow::Result<()> {
        match operation {
            MicrophoneOp::Mute => {
                if Self::exec(MUTE_COMMAND)?.stderr.is_empty() {
                    Err(anyhow::anyhow!("Error: Could not mute your microphone"))
                } else {
                    Ok(())
                }
            }
        }
    }
}
#[derive(Serialize, Deserialize, Copy, Clone, ValueEnum)]
pub enum MicrophoneOp {
    /// Toggle mute
    Mute,
}
