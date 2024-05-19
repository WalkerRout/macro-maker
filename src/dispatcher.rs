use std::path::Path;
use std::sync::mpsc;

use crate::manager::{Config, Manager};
use crate::process::{Process, Processor};
use crate::transmit::{Transmit, Transmitter};
use crate::Script;

/// Handles registering a file and starting the event loop
pub struct Dispatcher<P, T> {
  manager: Manager<Config>,
  processor: P,
  transmitter: T,
}

impl<P, T> Dispatcher<P, T>
where
  P: Process,
  T: Transmit,
{
  pub fn listen(&mut self) {
    let _phandle = self.processor.process_incoming_scripts();
    self.transmitter.listen_for_hotkeys(&self.manager);
  }
}

impl Dispatcher<Processor, Transmitter> {
  pub fn from_path<F>(path: F) -> Result<Self, anyhow::Error>
  where
    F: AsRef<Path>,
  {
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
}
