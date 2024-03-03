use crate::acpi::Acpi;
use crate::args::Command;
use crate::battery::Battery;
use crate::brightness::Brightness;
use crate::config::Config;
use crate::hypr::Hypr;
use crate::system::System;
use crate::volume::Volume;
use crate::wifi::Wifi;
use anyhow::anyhow;

pub struct Manager;

impl Manager {
    pub fn daemon(config: &Config) -> anyhow::Result<()> {
        if Self::running() {
            return Err(anyhow!("Manager is already running"));
        }
        Self::init_view(config)?;
        std::thread::scope(|scope| -> anyhow::Result<()> {
            let handle = scope.spawn(|| Acpi::listen(config));
            scope.spawn(|| Battery::listen());
            scope.spawn(|| Hypr::listen());
            scope.spawn(|| Wifi::listen());

            handle.join().unwrap()
        })
    }

    fn init_view(config: &Config) -> anyhow::Result<()> {
        config.init_view()?;
        Volume::init_view()?;
        Brightness::init_view()?;
        Wifi::init_view()?;
        Battery::init_view()?;
        Ok(())
    }

    pub fn handle(command: Command, config: &Config) -> anyhow::Result<()> {
        match command {
            Command::System { operation } => System::handle(operation),
            Command::Brightness { operation } => Brightness::handle(operation),
            Command::Volume { operation } => Volume::handle(operation),
            Command::Layout { operation } => Hypr::change_layout(operation, config),
            Command::Wifi { operation } => Wifi::handle(operation),
            Command::Daemon => Ok(()),
        }
    }

    pub fn running() -> bool {
        let pgrep = std::process::Command::new("pgrep")
            .args(["-f", "emanager daemon"])
            .output();
        pgrep.is_ok_and(|output| {
            String::from_utf8(output.stdout).is_ok_and(|stdout| stdout.lines().count() > 1)
        })
    }
}
