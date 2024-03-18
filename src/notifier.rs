use crate::{
    config::{CompositorType, CurrentState},
    logger::Logger,
};
use notify_rust::{Hint, Notification, Urgency};

pub struct Notifier {
    logger: Logger<u32>,
}

impl Notifier {
    pub fn new(name: &str) -> Self {
        Self {
            logger: Logger::new(&format!("{name}.id")),
        }
    }

    pub fn send(
        &self,
        summary: &str,
        body: &str,
        urgency: Option<Urgency>,
        value: Option<u32>,
    ) -> anyhow::Result<()> {
        if CompositorType::is_a_compositor_running() {
            let current_state: CurrentState = CompositorType::get_current_state()?;
            let color = format!("#{}ee", current_state.color);
            let mut notif = Notification::new()
                .summary(summary)
                .body(body)
                .hint(Hint::Urgency(urgency.unwrap_or(Urgency::Normal)))
                .hint(Hint::Custom("frcolor".to_string(), color))
                .finalize();
            if let Some(value) = value {
                notif = notif
                    .hint(Hint::CustomInt("value".to_string(), value as i32))
                    .finalize();
            }
            let id = if let Ok(id) = self.logger.read() {
                notif.id(id).show()?.id()
            } else {
                notif.show()?.id()
            };
            self.logger.write(&id)?;
        }
        Ok(())
    }
}
