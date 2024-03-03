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
        let layouts = self.layouts.iter().fold(
            (
                Vec::with_capacity(self.layouts.len()),
                Vec::with_capacity(self.layouts.len()),
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
            layout_to_set.layout.to_owned() + "," + &layouts.0.join(","),
            layout_to_set
                .variant
                .as_ref()
                .map_or(String::new(), String::to_owned)
                + ","
                + &layouts.1.join(","),
        );
        let current = match current_layout {
            Some(v) => v.to_owned(),
            None => Layout::try_from_keyword()?,
        };

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layouts: vec![Layout::new("fr", None), Layout::new("us", None)],
        }
    }
}
