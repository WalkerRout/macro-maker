use std::path::Path;
use std::sync::mpsc;

use crate::manager::{Config, Manager};
use crate::processor::Processor;
use crate::transmitter::Transmitter;
use crate::Script;

/// Handles registering a file and starting the event loop
pub struct Dispatcher {
  manager: Manager<Config>,
  processor: Processor,
  transmitter: Transmitter,
}

impl Dispatcher {
  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
    let (script_tx, script_rx) = mpsc::channel::<Script>();

    let manager = Manager::with_path(path)?;
    let processor = Processor::with_receiver(script_rx);
    let transmitter = Transmitter::with_sender(script_tx);

    Ok(Self {
      manager,
      processor,
      transmitter,
    })
  }

  pub fn listen(&mut self) {
    let _processor_handle = self.processor.spin();
    self.transmitter.spin(&self.manager);
  }
}
