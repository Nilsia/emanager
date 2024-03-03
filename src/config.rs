use std::path::Path;

use hyprland::keyword::Keyword;
use serde::{Deserialize, Serialize};

use crate::{layout::Layout, logger::Logger};

#[derive(Deserialize, Clone, Debug, Serialize)]
#[serde(default)]
pub struct Config {
    pub layouts: Vec<Layout>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        if path.as_ref().exists() {
            let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;
            // TODO check if layouts are valid
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub(crate) fn set_eww(
        &self,
        layout_to_set: &Layout,
        current_layout: Option<&Layout>,
    ) -> anyhow::Result<()> {
        if self.layouts.len() == 1 {
            return Ok(());
        }
        let current = match current_layout {
            Some(v) => v.to_owned(),
            None => Layout::try_from_keyword()?,
        };
        let (layouts, layouts_var) = self.layouts.iter().fold(
            (
                Vec::with_capacity(self.layouts.len()), // layout
                Vec::with_capacity(self.layouts.len()), // variant
            ),
            |(mut lay, mut var), l| {
                if l != layout_to_set {
                    lay.push(l.layout.to_owned());
                    var.push(l.variant.as_ref().map_or(String::new(), String::to_owned));
                }
                (lay, var)
            },
        );

        let (layout_str, layout_variant_str) = (
            self.generate_layout_hypr(layout_to_set, &layouts),
            self.generate_variant_hypr(layout_to_set, &layouts_var),
        );

        // this allow smooth changes between layouts (Hyprland were crashing from layout without variants to layout with variants (resp reverse))
        if current.layout != layout_to_set.layout {
            match (&layout_to_set.variant, &current.variant) {
                (None, Some(_)) => {
                    self.set_eww(&Layout::new(&current.layout, None), Some(&current))?
                }
                (Some(_), None) => {
                    self.set_eww(&Layout::new(&layout_to_set.layout, None), Some(&current))?
                }
                (Some(_), Some(_)) => {
                    self.set_eww(&Layout::new(&layout_to_set.layout, None), Some(&current))?
                }
                (None, None) => (),
            }
        }

        Keyword::set("input:kb_layout", layout_str)?;
        Keyword::set("input:kb_variant", layout_variant_str)?;
        Ok(())
    }

    fn generate_variant_hypr(&self, layout_to_set: &Layout, layouts: &[String]) -> String {
        layout_to_set
            .variant
            .as_ref()
            .map_or(String::new(), String::to_owned)
            + ","
            + &layouts.join(",")
    }

    fn generate_layout_hypr(&self, layout_to_set: &Layout, layouts: &[String]) -> String {
        layout_to_set.layout.to_owned() + "," + &layouts.join(",")
    }

    pub(crate) fn send_to_eww(&self, layout: &Layout) -> anyhow::Result<()> {
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

    fn get_layout_sequence(&self) -> anyhow::Result<Vec<Layout>> {
        let data_err = Logger::<Vec<String>>::new("layouts_sequence").read();
        if let Err(e) = data_err.as_ref() {
            match e.downcast_ref::<std::io::Error>() {
                Some(e_) => match e_.kind() {
                    std::io::ErrorKind::NotFound => (),
                    _ => return Err(anyhow::anyhow!(e.to_string())),
                },
                None => return Err(anyhow::anyhow!(e.to_string())),
            }
        }
        Ok(data_err
            .unwrap()
            .iter()
            .flat_map(Layout::try_from)
            .collect::<Vec<Layout>>())
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
                self.update_layout_sequence(layout)?;
                self.set_eww(layout, None)?;
                self.send_to_eww(layout)?;
                Ok(())
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layouts: vec![Layout::new("fr", None), Layout::new("us", None)],
        }
    }
}
