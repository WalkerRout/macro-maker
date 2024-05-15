use std::mem;
use std::path::Path;
use std::sync::mpsc;

use global_hotkey::GlobalHotKeyManager;

use crate::config::{exit_key, KeyIdMap, KeyMap};
use crate::processor::Processor;
use crate::transmitter::Transmitter;

// handle dispatching between keybinds and scripts\
pub struct Dispatcher {
  key_map: KeyIdMap,
  _manager: GlobalHotKeyManager,
}

impl Dispatcher {
  pub fn new(key_map: KeyIdMap, _manager: GlobalHotKeyManager) -> Self {
    // remove exit key if accidentally registered by user
    if _manager.unregister(exit_key()).is_ok() {
      log::warn!("do not register Ctrl+Shift+Alt+KeyE; it is the built in exit key");
    }
    // re-register exit key for use by system
    let _ = _manager.register(exit_key());
    Self { key_map, _manager }
  }

  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
    let key_map = KeyMap::from_file(path)?;
    let manager = GlobalHotKeyManager::new()?;
    for key in key_map.keys() {
      let _ = manager.register(*key);
    }
    Ok(Self::new(key_map.into(), manager))
  }

  pub fn listen(&mut self) {
    let (tx, rx) = mpsc::channel::<String>();
    let mut processor = Processor::new(rx);
    let _handle = processor.spawn(); // ProcessorHandle
    Transmitter::new(tx).spin(mem::take(&mut self.key_map));
  }
}
