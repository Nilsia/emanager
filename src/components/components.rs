use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::logger::Logger;

pub trait ScaledComponent<T: NotifiableState> {
    fn get() -> anyhow::Result<u32>;
    fn set(percent: u32) -> anyhow::Result<()>;
    fn up(percent: u32) -> anyhow::Result<()>;
    fn down(percent: u32) -> anyhow::Result<()>;
    fn get_state() -> anyhow::Result<T>;

    fn update(delay: u64) -> anyhow::Result<()> {
        if delay != 0 {
            std::thread::sleep(Duration::from_millis(delay));
        }
        let state = Self::get_state()?;
        state.notify()?;
        state.update_view()
    }
    fn init_view() -> anyhow::Result<()> {
        Self::get_state()?.update_view()
    }
}

pub trait NotifiableState: Serialize + for<'a> Deserialize<'a> {
    fn json_name(&self) -> &str;
    fn notify(&self) -> anyhow::Result<()>;
    fn update_view(&self) -> anyhow::Result<()> {
        Logger::new(self.json_name()).send(self)
    }
}
