use crate::acpi::Acpi;
use crate::args::Command;
use crate::components::components::ScaledComponent;
use crate::components::{
    battery::Battery, brightness::Brightness, microphone::Microphone, system::System,
    volume::Volume, wifi::Wifi,
};
use crate::compositors::{compositor::Compositor, hypr::Hypr, niri::Niri};
use crate::config::Config;
use anyhow::anyhow;

pub struct Manager;

impl Manager {
    pub fn daemon(config: &Config) -> anyhow::Result<()> {
        if Self::running() {
            return Err(anyhow!("Manager is already running"));
        }
        let seq = config.get_layout_sequence()?;
        let current_layout = config.compositor_type.get_first_layout_sequence()?;
        config.set_layout(
            &seq.first().expect("Error: sequence is empty"), // should never occure
            Some(&current_layout),
        )?;
        Self::init_view(config)?;
        std::thread::scope(|scope| -> anyhow::Result<()> {
            let handle = scope.spawn(|| Acpi::listen(config));
            scope.spawn(|| Battery::listen());
            match config.compositor_type {
                crate::config::CompositorType::Hyprland => scope.spawn(|| Hypr::listen()),
                crate::config::CompositorType::Niri => scope.spawn(|| Niri::listen()),
            };
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
        Microphone::init_view()?;
        Ok(())
    }

    pub fn handle(command: Command, config: &Config) -> anyhow::Result<()> {
        match command {
            Command::System { operation } => System::handle(operation),
            Command::Brightness { operation } => Brightness::handle(operation),
            Command::Volume { operation } => Volume::handle(operation),
            Command::Layout { operation } => config.change_layout(operation),
            Command::Wifi { operation } => Wifi::handle(operation),
            Command::Microphone { operation } => Microphone::handle(operation),
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
