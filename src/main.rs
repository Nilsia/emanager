use clap::Parser;

mod acpi;
mod args;
mod battery;
mod brightness;
mod config;
mod hypr;
pub mod layout;
mod logger;
mod manager;
mod notifier;
mod system;
mod utils;
mod volume;
mod wifi;

fn main() {
    let args = args::Args::parse();
    let config = dirs::home_dir().map_or(Ok(config::Config::default()), |home| {
        config::Config::from_file(home.join(".config/emanager/config.toml"))
    });
    if let Err(e) = config {
        eprintln!("{e}");
        return;
    }
    let config = config.unwrap();

    let result = match args.command {
        args::Command::Daemon => manager::Manager::daemon(&config),
        _ => manager::Manager::handle(args.command, &config),
    };

    if let Err(e) = result {
        eprintln!("{e}");
    }
}
