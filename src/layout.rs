use std::fmt::Display;

use hyprland::keyword::Keyword;
use serde::{Deserialize, Serialize};

use crate::logger::Logger;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Layout {
    pub layout: String,
    pub variant: Option<String>,
}

impl Layout {
    pub fn new(layout: &str, variant: Option<&str>) -> Self {
        Self {
            layout: layout.to_string(),
            variant: variant.map(String::from),
        }
    }

    pub(crate) fn send_to_eww(&self, layouts: &[Layout]) -> anyhow::Result<()> {
        Logger::new("layout-list").send(
            &layouts
                .iter()
                .filter(|l| l != &self)
                .map(Self::to_string)
                .collect::<Vec<String>>(),
        )?;
        Logger::new("layout-selected").send(self)
    }

    pub(crate) fn to_string(&self) -> String {
        let mut s = self.layout.to_owned();
        if let Some(variant) = self.variant.as_ref() {
            s += ":";
            s += variant;
        }
        s
    }

    pub(crate) fn try_from_keyword() -> anyhow::Result<Self> {
        let var_string = Keyword::get("input:kb_variant")?.value.to_string();
        let lay_string = Keyword::get("input:kb_layout")?.value.to_string();
        let var: Vec<&str> = var_string.split(",").collect();
        let lay: Vec<&str> = lay_string.split(",").collect();
        if var.is_empty() || lay.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid layouts configuration please fill kb_layout and kb_variant"
            ));
        }
        Ok(Self::new(
            lay.get(0).unwrap(),
            var.first()
                .map_or(None, |v| if v.trim().is_empty() { None } else { Some(v) }),
        ))
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.layout)?;
        if let Some(variant) = self.variant.as_ref() {
            write!(f, ":{}", variant)?;
        }
        Ok(())
    }
}

impl TryFrom<&str> for Layout {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(anyhow::anyhow!("Please provide non empty layout"));
        }
        let splitted: Vec<&str> = value.split(":").collect();
        if splitted.len() > 2 {
            return Err(anyhow::anyhow!("Invalid layout"));
        }
        Ok(Self {
            layout: splitted.get(0).unwrap().to_string(),
            variant: splitted.get(1).map(|s| s.to_string()),
        })
    }
}

impl TryFrom<&String> for Layout {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(anyhow::anyhow!("Please provide non empty layout"));
        }
        let splitted: Vec<&str> = value.split(":").collect();
        if splitted.len() > 2 {
            return Err(anyhow::anyhow!("Invalid layout"));
        }
        Ok(Self {
            layout: splitted.get(0).unwrap().to_string(),
            variant: splitted.get(1).map(|s| s.to_string()),
        })
    }
}

impl Serialize for Layout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, clap::Subcommand)]
pub enum LayoutOp {
    Set { layout: String },
    Switch,
}
