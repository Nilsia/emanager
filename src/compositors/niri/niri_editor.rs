use std::{fs::OpenOptions, io::Write};

use kdl::{KdlDocument, KdlEntry, KdlIdentifier, KdlNode, KdlValue};

pub struct NiriEditor {}

impl NiriEditor {
    fn get_config() -> anyhow::Result<KdlDocument> {
        let Some(dirs) = directories::ProjectDirs::from("", "", "niri") else {
            return Err(anyhow::anyhow!("Error while retrieving your niri configuration file, make sure to have your configuration file at the right place."));
        };
        let mut path = dirs.config_dir().to_owned();
        path.push("config.kdl");
        Ok(std::fs::read_to_string(&path)?.parse::<KdlDocument>()?)
    }

    fn insert_into_document<T: Into<KdlValue> + Clone, K: ToString + Into<KdlIdentifier>>(
        doc: &mut KdlDocument,
        paths: &[impl ToString],
        key: K,
        value: Option<&[T]>,
    ) -> anyhow::Result<()> {
        if paths.len() == 0 {
            let node = if let Some(n) = doc.get_mut(&key.to_string()) {
                n.clear_entries();
                n
            } else {
                doc.nodes_mut().push(KdlNode::new(key));
                doc.nodes_mut().last_mut().unwrap()
            };
            if let Some(v) = value {
                node.entries_mut()
                    .extend(v.iter().map(|l| KdlEntry::new(l.to_owned())))
            }
        } else {
            let path = paths.first().unwrap().to_string();
            match doc.get_mut(&path) {
                Some(n) => {
                    Self::insert_into_document(n.ensure_children(), &paths[1..], key, value)?
                }
                None => {
                    doc.nodes_mut().push(KdlNode::new(path));
                    Self::insert_into_document(
                        doc.nodes_mut().last_mut().unwrap().ensure_children(),
                        &paths[1..],
                        key,
                        value,
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn set<T: Into<KdlValue> + Clone, K: ToString + Into<KdlIdentifier>>(
        paths: &[impl ToString],
        key: K,
        value: Option<&[T]>,
    ) -> anyhow::Result<()> {
        let mut config = Self::get_config()?;
        // let mut config: KdlDocument = std::fs::read_to_string("cc.kdl")?.parse()?;
        Self::insert_into_document(&mut config, paths, key, value)?;
        OpenOptions::new()
            .write(true)
            .create(true)
            .open("test.kdl")?
            .write_all(config.to_string().as_bytes())?;
        Ok(())
    }

    pub fn get(
        paths: &[impl ToString],
        key: impl ToString,
    ) -> anyhow::Result<Option<Vec<KdlValue>>> {
        let mut config = &Self::get_config()?;
        // let mut config: &KdlDocument = &std::fs::read_to_string("cc.kdl")?.parse()?;
        let mut p: String;
        for path in paths {
            p = path.to_string();
            config = match config.get(&p) {
                Some(node) => match node.children() {
                    Some(doc) => doc,
                    None => {
                        return Err(anyhow::anyhow!(
                            "Could not find next of node named '{}'.",
                            &p
                        ));
                    }
                },
                None => {
                    return Err(anyhow::anyhow!("Could not find node named '{}'.", &p));
                }
            };
        }
        Ok(config.get(&key.to_string()).map(|node| {
            node.entries()
                .iter()
                .map(|e| e.value().to_owned())
                .collect()
        }))
    }
}
