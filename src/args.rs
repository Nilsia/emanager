use crate::system::SystemOp;
use crate::volume::VolumeOp;
use crate::{brightness::BrightnessOp, wifi::WifiTurnType};
use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Launch manager daemon
    Daemon,
    /// Commands to manage systemd
    System {
        #[command(subcommand)]
        operation: SystemOp,
    },
    /// Commands to manage backlight
    Brightness {
        #[command(subcommand)]
        operation: BrightnessOp,
    },
    /// Commands to manage volume
    Volume {
        #[command(subcommand)]
        operation: VolumeOp,
    },
    /// Change layout
    Layout { layout: String },
    /// Commands to handle wifi
    Wifi {
        #[arg(value_enum)]
        operation: WifiTurnType,
    },
}
