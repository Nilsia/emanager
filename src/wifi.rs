use crate::logger::Logger;
use crate::notifier::Notifier;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::Display;
use std::process::{Command, Output};
use std::time::Duration;

const PROGRAM: &str = "nmcli";

pub struct Wifi;

impl Wifi {
    pub fn listen() -> anyhow::Result<()> {
        let mut current = None;
        loop {
            let state = Self::get_state()?;
            if Some(&state) != current.as_ref() {
                state.notify_connection_update(current)?;
                state.log()?;
                current = Some(state);
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    fn get_state() -> anyhow::Result<WifiState> {
        let output = Self::exec(&["-t", "-f", "active,ssid,signal", "dev", "wifi"])?;
        Ok(String::from_utf8(output.stdout)?
            .lines()
            .map(|line| {
                let info = line.split(':').collect::<Vec<&str>>();
                let active = info.get(0).is_some_and(|i| i == &"yes");
                let ssid = info.get(1).unwrap_or(&"");
                let signal = info.get(2).unwrap_or(&"0").parse::<u32>().unwrap_or(0);
                WifiState::new(active, ssid, signal)
            })
            .find(|state| state.active)
            .unwrap_or(WifiState::new(false, "", 0)))
    }

    fn turn_on() -> anyhow::Result<()> {
        Self::turn(WifiTurnType::On)
    }

    fn turn_off() -> anyhow::Result<()> {
        Self::turn(WifiTurnType::Off)
    }

    /// return
    fn turn(signal: WifiTurnType) -> anyhow::Result<()> {
        let (state, error) = if Self::exec(&["radio", "wifi", signal.as_ref()])?
            .stderr
            .is_empty()
        {
            (Self::get_state()?, false)
        } else {
            (WifiState::new(false, "", 0), true)
        };
        state.log()?;
        state.notify_switch_update((WifiTurnType::On, error))
    }

    fn exec(args: &[impl AsRef<OsStr>]) -> anyhow::Result<Output> {
        let output = Command::new(PROGRAM).args(args).output()?;
        Ok(output)
    }

    pub fn handle(operation: WifiTurnType) -> anyhow::Result<()> {
        match operation {
            WifiTurnType::On => Self::turn_on(),
            WifiTurnType::Off => Self::turn_off(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
struct WifiState {
    active: bool,
    ssid: String,
    icon: String,
}

impl WifiState {
    pub fn new(active: bool, ssid: &str, signal: u32) -> Self {
        let icon = if !active {
            "󰤭 "
        } else if signal >= 75 {
            "󰤨 "
        } else if signal >= 50 {
            "󰤥 "
        } else if signal >= 25 {
            "󰤢 "
        } else {
            "󰤟 "
        }
        .to_string();
        let ssid = ssid.to_string();
        Self { active, ssid, icon }
    }

    pub fn notify_connection_update(&self, prev: Option<Self>) -> anyhow::Result<()> {
        let notifier = Notifier::new("wifi");
        if self.active && !prev.as_ref().is_some_and(|s| s.active) {
            notifier.send(
                "Wi-Fi",
                &format!("Connected to '{}'", self.ssid),
                None,
                None,
            )?;
        } else if !self.active && prev.as_ref().is_some_and(|s| s.active) {
            notifier.send(
                "Wi-Fi",
                &format!("Disconnected from '{}'", prev.unwrap().ssid),
                None,
                None,
            )?;
        }
        Ok(())
    }

    pub fn notify_switch_update(
        &self,
        (operation, error): (WifiTurnType, bool),
    ) -> anyhow::Result<()> {
        let notifier = Notifier::new("wifi");
        if error {
            notifier.send(
                "Wi-Fi",
                &format!("Unable to turn {} wifi", operation),
                None,
                None,
            )?;
        } else {
            notifier.send(
                "Wi-Fi",
                &format!("Turned {} the wifi", operation),
                None,
                None,
            )?;
        }
        Ok(())
    }

    pub fn log(&self) -> anyhow::Result<()> {
        Logger::new("wifi").write(self)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, ValueEnum)]
pub enum WifiTurnType {
    On,
    Off,
}

impl Display for WifiTurnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for WifiTurnType {
    fn as_ref(&self) -> &str {
        match self {
            WifiTurnType::On => "on",
            WifiTurnType::Off => "off",
        }
    }
}
