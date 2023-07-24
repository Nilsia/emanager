use clap::Subcommand;
use serde::Serialize;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use zbus::{blocking::Connection, zvariant::DynamicType, Message};

#[derive(Clone)]
pub struct System;

impl System {
    pub fn poweroff() -> anyhow::Result<()> {
        Self::call("poweroff", &true)?;
        Ok(())
    }

    pub fn reboot() -> anyhow::Result<()> {
        Self::call("Reboot", &true)?;
        Ok(())
    }

    pub fn suspend() -> anyhow::Result<()> {
        Self::call("Suspend", &true).and_then(|_| Self::lock())
    }

    pub fn lock() -> anyhow::Result<()> {
        Command::new("pkill").arg("swaylock").output()?;
        Command::new("swaylock").arg("-f").output()?;
        Ok(())
    }

    pub fn inhibit() -> anyhow::Result<()> {
        let _handle = Self::call("Inhibit", &("idle", "emanager", "Idle inhibitor", "block"))?;
        loop {
            std::thread::sleep(Duration::from_secs(2_u64.pow(8)));
        }
    }

    pub fn handle(operation: SystemOp) -> anyhow::Result<()> {
        match operation {
            SystemOp::Poweroff => Self::poweroff(),
            SystemOp::Reboot => Self::reboot(),
            SystemOp::Suspend => Self::suspend(),
            SystemOp::Lock => Self::lock(),
            SystemOp::Inhibit => Self::inhibit(),
        }
    }

    fn call(method: &str, body: &(impl Serialize + DynamicType)) -> anyhow::Result<Arc<Message>> {
        let bus = Connection::system()?;
        let message = bus.call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            method,
            body,
        )?;
        Ok(message)
    }
}

#[derive(Copy, Clone, Subcommand)]
pub enum SystemOp {
    /// Turn system off
    Poweroff,
    /// Reboot system
    Reboot,
    /// Suspend system
    Suspend,
    /// Lock system
    Lock,
    /// Inhibit idle
    Inhibit,
}
