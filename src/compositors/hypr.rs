use crate::config::CompositorType;
use crate::layout::Layout;
use crate::logger::Logger;
use hyprland::data::{Client, Workspace, Workspaces};
use hyprland::keyword::Keyword;
use hyprland::shared::{HyprData, HyprDataActive, HyprDataActiveOptional};

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::time::Duration;

use super::compositor::Compositor;

pub struct Hypr;

impl Compositor for Hypr {
    fn listen() -> anyhow::Result<()> {
        while !Self::running() {
            std::thread::sleep(Duration::from_secs(1));
        }

        Self::change_workspace()?;

        let stream = Self::stream()?;
        let reader = BufReader::new(stream);
        let mut current = Self::get_active_address()?;
        for line in reader.lines().flatten() {
            let address = Self::get_active_address()?;
            if line.starts_with("workspace") {
                Self::change_workspace()?;
            } else if line.starts_with("activewindowv2") && address != current {
                Self::change_color()?;
                current = address;
            }
        }

        Ok(())
    }

    fn running() -> bool {
        let pgrep = Command::new("pgrep").arg("Hyprland").output();
        pgrep.is_ok_and(|output| !output.stdout.is_empty())
    }

    fn change_color() -> anyhow::Result<()> {
        let color = Self::rand_color();
        Keyword::set("general:col.active_border", format!("rgba({color}ee)"))?;
        Self::send_color_to_view(&color)?;
        Ok(())
    }

    fn get_corresponding_compositor_type() -> crate::config::CompositorType {
        CompositorType::Hyprland
    }

    fn set_layout(layout_to_set: &Layout, config_layouts: &[Layout]) -> anyhow::Result<()> {
        let (layouts, layouts_var) = config_layouts.iter().fold(
            (
                Vec::with_capacity(config_layouts.len()), // layout
                Vec::with_capacity(config_layouts.len()), // variant
            ),
            |(mut lay, mut var), l| {
                if l != layout_to_set {
                    lay.push(l.layout.to_owned());
                    var.push(l.variant.as_ref().map_or(String::new(), String::to_owned));
                }
                (lay, var)
            },
        );
        Keyword::set(
            "input:kb_layout",
            Layout::generate_layout_sequence(layout_to_set, &layouts),
        )?;
        Keyword::set(
            "input:kb_variant",
            Layout::generate_variant_sequence(layout_to_set, &layouts_var),
        )?;
        Ok(())
    }

    fn get_first_layout_sequence() -> anyhow::Result<Layout> {
        let layouts: Vec<String> = match Keyword::get("input:kb_layout")?.value {
            hyprland::keyword::OptionValue::String(v) => v.split(',').map(String::from).collect(),
            _ => return Err(anyhow::anyhow!("Invalid layouts")),
        };
        let variants: Vec<String> = match Keyword::get("input:kb_variant")?.value {
            hyprland::keyword::OptionValue::String(v) => v.split(',').map(String::from).collect(),
            _ => vec![String::new()],
        };
        Ok(Layout::new(
            layouts
                .first()
                .ok_or(anyhow::anyhow!("Error: first layout not found"))?,
            variants.first().map(|v| v.as_str()),
        ))
    }
}

impl Hypr {
    fn get_active_address() -> anyhow::Result<Option<Vec<u8>>> {
        Ok(Client::get_active()?.map(|client| client.address.as_vec()))
    }

    fn stream() -> anyhow::Result<UnixStream> {
        let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        Ok(UnixStream::connect(format!(
            "/tmp/hypr/{signature}/.socket2.sock"
        ))?)
    }
    pub fn change_workspace() -> anyhow::Result<()> {
        let mut states = Workspaces::get()?
            .flat_map(WorkspaceState::try_from)
            .collect::<Vec<WorkspaceState>>();
        for id in 1..=5 {
            if states.iter().all(|state| state.id != id) {
                states.push(WorkspaceState::new(id, 0, false));
            }
        }
        states.sort_by_key(|workspace| workspace.id);
        Logger::new("workspaces-json").send(&states)
    }
}

#[derive(Serialize, Deserialize)]
pub struct WorkspaceState {
    id: i32,
    windows: u16,
    active: bool,
}

impl TryFrom<Workspace> for WorkspaceState {
    type Error = anyhow::Error;
    fn try_from(value: Workspace) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.id,
            value.windows,
            value.id == Workspace::get_active()?.id,
        ))
    }
}

impl WorkspaceState {
    pub fn new(id: i32, windows: u16, active: bool) -> Self {
        Self {
            id,
            windows,
            active,
        }
    }
}
