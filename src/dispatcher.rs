use std::path::Path;
use std::sync::mpsc;

use crate::manager::{Config, Manager};
use crate::processor::Processor;
use crate::transmitter::Transmitter;
use crate::{Process, Script, Transmit};

/// Handles registering a file and starting the event loop
pub struct Dispatcher<P, T> {
  manager: Manager<Config>,
  processor: P,
  transmitter: T,
}

pub type DefaultDispatcher = Dispatcher<Processor, Transmitter>;

impl<P, T> Dispatcher<P, T>
where
  P: Process,
  T: Transmit,
{
  pub fn from_path<F>(path: F) -> Result<DefaultDispatcher, anyhow::Error>
  where
    F: AsRef<Path>,
  {
    let (script_tx, script_rx) = mpsc::channel::<Script>();

    let manager = Manager::with_path(path)?;
    let processor = Processor::with_receiver(script_rx);
    let transmitter = Transmitter::with_sender(script_tx);

    Ok(DefaultDispatcher {
      manager,
      processor,
      transmitter,
    })
  }

  pub fn listen(&mut self) {
    let _phandle = self.processor.process_incoming_scripts();
    self.transmitter.listen_for_hotkeys(&self.manager);
  }
}

/*



*/
