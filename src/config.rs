use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    compositors::{compositor::Compositor, hypr::Hypr, niri::Niri},
    layout::{Layout, LayoutOp},
    logger::Logger,
};

const LAYOUT_SEQUENCE_FILENAME: &str = "layouts_sequence";

pub struct CurrentState {
    pub color: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub enum CompositorType {
    Hyprland,
    Niri,
}
impl CompositorType {
    fn set_layout(&self, layout_to_set: &Layout, config_layouts: &[Layout]) -> anyhow::Result<()> {
        match self {
            CompositorType::Hyprland => Hypr::set_layout(layout_to_set, config_layouts),
            CompositorType::Niri => Niri::set_layout(layout_to_set, config_layouts),
        }
    }

    pub(crate) fn get_first_layout_sequence(&self) -> anyhow::Result<Layout> {
        match self {
            CompositorType::Hyprland => Hypr::get_first_layout_sequence(),
            CompositorType::Niri => Niri::get_first_layout_sequence(),
        }
    }

    pub(crate) fn get_running_compositor_type() -> anyhow::Result<Self> {
        if crate::compositors::hypr::Hypr::running() {
            Ok(Self::Hyprland)
        } else if crate::compositors::niri::Niri::running() {
            Ok(Self::Niri)
        } else {
            Err(anyhow::anyhow!("Could not find your compositor"))
        }
    }

    pub(crate) fn is_a_compositor_running() -> bool {
        crate::compositors::hypr::Hypr::running() || crate::compositors::niri::Niri::running()
    }

    pub(crate) fn get_current_state() -> anyhow::Result<CurrentState> {
        Self::get_running_compositor_type().map(|v| match v {
            CompositorType::Hyprland => CurrentState {
                color: Hypr::get_color(),
            },
            CompositorType::Niri => CurrentState {
                color: Niri::get_color(),
            },
        })
    }
}

impl Default for CompositorType {
    fn default() -> Self {
        Self::get_running_compositor_type().unwrap()
    }
}

#[derive(Deserialize, Clone, Debug, Serialize)]
#[serde(default)]
pub struct Config {
    pub layouts: Vec<Layout>,
    pub compositor_type: CompositorType,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        if path.as_ref().exists() {
            let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;
            if !Logger::<bool>::new(LAYOUT_SEQUENCE_FILENAME).try_exists()? {
                config.init_layout_sequence()?;
            }
            // TODO check if layouts are valid
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub(crate) fn set_layout(
        &self,
        layout_to_set: &Layout,
        current_layout: Option<&Layout>,
    ) -> anyhow::Result<()> {
        if self.layouts.len() == 1 {
            return Ok(());
        }
        let current = match current_layout {
            Some(v) => v.to_owned(),
            None => Layout::try_from_sequence()?,
        };

        match self.compositor_type {
            CompositorType::Hyprland => {
                // this allow smooth changes between layouts (Hyprland were crashing from layout without variants to layout with variants (resp reverse))
                if current.layout != layout_to_set.layout {
                    match (&layout_to_set.variant, &current.variant) {
                        (None, Some(_)) => {
                            self.set_layout(&Layout::new(&current.layout, None), Some(&current))?
                        }
                        (Some(_), None) => self.set_layout(
                            &Layout::new(&layout_to_set.layout, None),
                            Some(&current),
                        )?,
                        (Some(_), Some(_)) => self.set_layout(
                            &Layout::new(&layout_to_set.layout, None),
                            Some(&current),
                        )?,
                        (None, None) => (),
                    }
                }
            }
            CompositorType::Niri => (),
        }

        self.compositor_type
            .set_layout(layout_to_set, &self.layouts)?;
        Ok(())
    }

    pub(crate) fn send_to_view(&self, layout: &Layout) -> anyhow::Result<()> {
        Logger::new("layout-list").send(
            &self
                .layouts
                .iter()
                .filter(|l| l != &layout)
                .map(Layout::to_string)
                .collect::<Vec<String>>(),
        )?;
        Logger::new("layout-selected").send(layout)?;
        Ok(())
    }

    /// Returns the layouts sequence stored in the file named `LAYOUT_SEQUENCE_FILENAME`
    ///
    /// # Errors
    ///
    /// This function will return an error if there is any error excepted NotFound when reading the files containing the layouts sequence.
    pub fn get_layout_sequence(&self) -> anyhow::Result<Vec<Layout>> {
        let data_err = Logger::<Vec<String>>::new(LAYOUT_SEQUENCE_FILENAME).read();
        if let Err(e) = data_err.as_ref() {
            match e.downcast_ref::<std::io::Error>() {
                Some(e_) => match e_.kind() {
                    std::io::ErrorKind::NotFound => (),
                    _ => return Err(anyhow::anyhow!(e.to_string())),
                },
                None => return Err(anyhow::anyhow!(e.to_string())),
            }
        }
        if let Ok(data) = data_err {
            Ok(data
                .iter()
                .flat_map(Layout::try_from)
                .collect::<Vec<Layout>>())
        } else {
            Ok(Vec::new())
        }
    }

    fn init_layout_sequence(&self) -> anyhow::Result<()> {
        for layout in &self.layouts {
            self.update_layout_sequence(layout)?;
        }
        Ok(())
    }

    pub fn update_layout_sequence(&self, layout: &Layout) -> anyhow::Result<()> {
        let mut data = self.get_layout_sequence()?;
        data.retain(|l| l != layout);
        data.insert(0, layout.to_owned());
        Logger::new("layouts_sequence").overwrite(&data)?;

        Ok(())
    }

    pub fn switch_layout_sequence(&self) -> anyhow::Result<()> {
        let sequences = self.get_layout_sequence()?;
        match sequences.len() {
            0 | 1 => Ok(()),
            _ => {
                let layout = sequences.get(1).unwrap();
                self.set_layout(layout, None)?;
                self.send_to_view(layout)?;
                self.update_layout_sequence(layout)?;
                Ok(())
            }
        }
    }
    pub fn init_view(&self) -> anyhow::Result<()> {
        self.layouts
            .get(0)
            .map(|l| l.send_to_view(&self.layouts))
            .ok_or(anyhow::anyhow!(
                "Wrong configuration this should not happen (layouts) missing"
            ))??;
        Ok(())
    }

    pub fn change_layout(&self, operation: LayoutOp) -> anyhow::Result<()> {
        match operation {
            LayoutOp::Set { layout: layout_str } => {
                let layout = Layout::try_from(layout_str.as_str())?;
                if !self.layouts.contains(&layout) {
                    return Err(anyhow::anyhow!(
                        "Given layout '{layout}' does not exist in configuration"
                    ));
                }
                self.set_layout(&layout, None)?;
                self.send_to_view(&layout)?;
                self.update_layout_sequence(&layout)?;
                Ok(())
            }
            LayoutOp::Switch => self.switch_layout_sequence(),
            LayoutOp::Reset => {
                eprintln!("This feature is not working for now.");
                return Ok(());
                // self.change_layout(LayoutOp::Set {
                // layout: self.get_first_layout_of_sequence()?.to_string(),
                // })
            }
        }
    }

    #[allow(dead_code)]
    fn get_first_layout_of_sequence(&self) -> anyhow::Result<Layout> {
        self.get_layout_sequence()?
            .first()
            .ok_or(anyhow::anyhow!("Error: Cannot get layouts from sequence"))
            .map(|v| v.to_owned())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compositor_type: CompositorType::default(),
            layouts: vec![Layout::new("fr", None), Layout::new("us", None)],
        }
    }
}
