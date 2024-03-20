use std::process::Command;

use rand::seq::IteratorRandom;

use crate::{components::layout::Layout, config::CompositorType, logger::Logger};

const DEFAULT_COLOR: &str = "7aa2f7";
const COLORS: &[&str] = &["7aa2f7", "9ece6a", "e0af68", "bb9af7", "7dcfff", "c0caf5"];

pub trait Compositor {
    fn listen() -> anyhow::Result<()>;
    /// when creating this function, use Self::send_color_to_view
    fn change_color() -> anyhow::Result<()>;
    fn running() -> bool;
    fn set_layout(layout_to_set: &Layout, config_layouts: &[Layout]) -> anyhow::Result<()>;
    fn get_first_layout_sequence() -> anyhow::Result<Layout>;
    fn get_corresponding_compositor_type() -> CompositorType;

    fn get_color() -> String {
        // temporary fix because hyprctl doesn't work for colors
        Command::new("eww")
            .args(["get", "border-color"])
            .output()
            .map(|o| String::from_utf8(o.stdout).unwrap_or(DEFAULT_COLOR.to_string()))
            .unwrap_or(DEFAULT_COLOR.to_string())
    }

    fn rand_color() -> String {
        (*COLORS
            .iter()
            .filter(|color| (**color).to_string() != Self::get_color())
            .choose(&mut rand::thread_rng())
            .unwrap())
        .to_string()
    }

    fn send_color_to_view(color: &String) -> anyhow::Result<()> {
        Logger::new("border-color").send(color)
    }
}
