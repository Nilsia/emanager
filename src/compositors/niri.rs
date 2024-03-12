pub mod niri_editor;

use std::{process::Command, time::Duration};

use crate::{compositors::niri::niri_editor::NiriEditor, config::CompositorType, layout::Layout};

use super::compositor::Compositor;

pub const LAYOUT_NIRI_PATH: &[&str] = &["input", "keyboard", "xkb"];
const NIRI_CMD: &str = "niri";

pub struct Niri;

impl Compositor for Niri {
    fn listen() -> anyhow::Result<()> {
        while !Self::running() {
            std::thread::sleep(Duration::from_secs(1));
        }
        Ok(())
    }

    fn running() -> bool {
        let pgrep = Command::new("pgrep").arg("niri").output();
        pgrep.is_ok_and(|output| !output.stdout.is_empty())
    }

    fn change_color() -> anyhow::Result<()> {
        let color = Self::rand_color();
        Self::send_color_to_view(&color)?;
        NiriEditor::set(
            &vec!["layout", "focus-ring"],
            "active-color",
            Some(&vec![String::from("#") + &color]),
        )
    }

    fn get_corresponding_compositor_type() -> crate::config::CompositorType {
        CompositorType::Niri
    }

    fn set_layout(layout_to_set: &Layout, _: &[Layout]) -> anyhow::Result<()> {
        let available_layouts = Self::get_available_layouts()?;
        let current_layout = Layout::try_from_sequence()?;
        if current_layout == *layout_to_set {
            return Ok(());
        }
        let (mut index_to_set, mut current_index): (Option<usize>, Option<usize>) = (None, None);
        for (i, layout) in available_layouts.iter().enumerate() {
            if layout == layout_to_set {
                index_to_set = Some(i);
            } else if layout == &current_layout {
                current_index = Some(i);
            }
        }

        match (index_to_set, current_index) {
            (Some(s), Some(c)) => {
                let diff: isize = (c as isize - s as isize) / (-(s.abs_diff(c) as isize));
                let update_function = match diff {
                    -1 => || Self::change_layout("prev"),
                    1 => || Self::change_layout("next"),
                    _ => unreachable!(),
                };

                let mut i: isize = c as isize;
                while i as usize != s {
                    update_function()?;
                    i += diff;
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Error: Could not find current or requested layout in niri configuration"
            )),
        }
        // NiriEditor::set(&paths, "layout", Some(&vec![layout_str]))?;
        // NiriEditor::set(&paths, "variant", Some(&vec![layout_variant_str]))?;
        // Ok(())
    }

    fn get_first_layout_sequence() -> anyhow::Result<Layout> {
        let layouts = Self::get_available_layouts()?;
        layouts
            .first()
            .ok_or(anyhow::anyhow!(
                "Error: layouts in niri configuration are empty."
            ))
            .cloned()
    }
}

impl Niri {
    /// returns a vector of `Layout` following the layouts given inside the niri configuration
    ///
    /// # Errors
    ///
    /// This function will return an error if the data inside the niri configuration are not valid.
    fn get_available_layouts() -> anyhow::Result<Vec<Layout>> {
        let layouts = NiriEditor::get(LAYOUT_NIRI_PATH, "layout")?;
        let variants = NiriEditor::get(LAYOUT_NIRI_PATH, "variant")?;
        match layouts {
            Some(l) => match l.first() {
                Some(l) => {
                    let layouts: Vec<String> = l
                        .as_string()
                        .ok_or(anyhow::anyhow!("Invalid layouts in niri config"))?
                        .split(',')
                        .map(String::from)
                        .collect();
                    let variants =
                        variants.map_or(Ok(vec![String::new(); layouts.len()]), |v| {
                            v.first()
                                .map_or(Ok(vec![String::new(); layouts.len()]), |v| {
                                    v.as_string()
                                        .map(|v| {
                                            v.split(',').map(String::from).collect::<Vec<String>>()
                                        })
                                        .ok_or(anyhow::anyhow!("Invalid variants in niri config"))
                                })
                        })?;
                    if layouts.len() != variants.len() {
                        return Err(anyhow::anyhow!(
                            "Error: layouts and variants are incompatible"
                        ));
                    }
                    let lay: Vec<Layout> = layouts
                        .iter()
                        .zip(variants.iter())
                        .map(|(l, v)| Layout::new(l, if v.is_empty() { None } else { Some(v) }))
                        .collect();
                    let mut cloned = lay.clone();
                    cloned.sort();
                    cloned.dedup();
                    if cloned.len() != lay.len() {
                        return Err(anyhow::anyhow!("Error: same layout given twice"));
                    }

                    Ok(lay)
                }
                _ => Err(anyhow::anyhow!("Error: layouts present but empty")),
            },
            _ => Err(anyhow::anyhow!("Errro: could not get your layouts")),
        }
    }

    fn change_layout(direction: &str) -> anyhow::Result<()> {
        if Command::new(NIRI_CMD)
            .args(["msg", "action", "switch-layout", direction])
            .output()?
            .stderr
            .is_empty()
        {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Error: Could not change layout"))
        }
    }
}
