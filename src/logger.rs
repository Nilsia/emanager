use anyhow::anyhow;
use core::panic;
use serde::{Deserialize, Serialize};
use std::{io::Write, marker::PhantomData, path::PathBuf, process::Command};

const DIR: &str = ".local/state/emanager";

pub struct Logger<T: Serialize + for<'a> Deserialize<'a>> {
    full_path: PathBuf,
    file: String,
    name: String,
    phantom: PhantomData<T>,
}

impl<T: Serialize + for<'a> Deserialize<'a>> Logger<T> {
    pub fn new(name: &str) -> Self {
        let home_dir = dirs::home_dir();
        if home_dir.is_none() {
            panic!("Could not get your personnal directory");
        }
        let mut full_path = home_dir.unwrap();
        full_path.push(DIR);
        Self {
            name: name.to_string(),
            full_path: full_path.to_owned(),
            file: format!(
                "{}/{name}",
                full_path.to_str().expect("You home directory is not valid")
            ),
            phantom: PhantomData,
        }
    }

    pub fn send(&self, state: &T) -> anyhow::Result<()> {
        Command::new("eww")
            .args([
                "update",
                &format!("{}={}", self.name, serde_json::to_string(state)?),
            ])
            .output()?;
        Ok(())
    }

    pub fn write(&self, state: &T) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.full_path)?;
        self.truncate()?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)?;
        let json = serde_json::to_vec(&state)?;
        file.write_all(&json)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn read(&self) -> anyhow::Result<T> {
        let state = std::fs::read_to_string(&self.file)?
            .lines()
            .last()
            .map(String::from)
            .ok_or(anyhow!("State not found"))?;
        Ok(serde_json::from_str(&state)?)
    }

    fn truncate(&self) -> anyhow::Result<()> {
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.file)
        {
            if file.metadata()?.len() > 2_u64.pow(16) {
                file.set_len(0)?;
            }
        }
        Ok(())
    }

    pub fn overwrite(&self, state: &T) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.full_path)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.file)?;
        let json = serde_json::to_vec(&state)?;
        file.write_all(&json)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn try_exists(&self) -> anyhow::Result<bool> {
        Ok(PathBuf::from(&self.file).try_exists()?)
    }
}
