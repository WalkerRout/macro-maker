use std::mem;
use std::sync::mpsc::Sender;
use std::thread;

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use winit::event_loop::{ControlFlow, EventLoopBuilder, EventLoopWindowTarget};

use crate::manager::{exit_key, Manager};
use crate::{Dispatchable, Script};

#[derive(Debug)]
pub struct Transmitter {
  tx: Option<Sender<Script>>,
}

impl Transmitter {
  pub fn with_sender(tx: Sender<Script>) -> Self {
    Self { tx: Some(tx) }
  }

  pub fn spin<T>(&mut self, manager: &Manager<T>)
  where
    T: Dispatchable,
  {
    let hotkey_manager = GlobalHotKeyManager::new().expect("global hotkey manager");
    hotkey_manager_register_keys(&hotkey_manager, &manager.hotkeys());

    let event_loop = EventLoopBuilder::new()
      .build()
      .expect("event loop spun twice");
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    let mut terminate = false;

    event_loop
      .run(move |_event, event_loop| {
        if terminate {
          event_loop.exit();
          return;
        }
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Ok(()) = manager.try_update() {
          let hotkeys = manager.hotkeys();
          hotkey_manager_register_keys(&hotkey_manager, &hotkeys);
          log::info!("reloaded hotkey manager");
        }
        if let Ok(event) = global_hotkey_channel.try_recv() {
          terminate = !self.process_event(event, event_loop, manager);
        }
        // avoid spinning and eating up cpu
        thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
      })
      .expect("run event loop");
    log::info!("Transmitter terminated");
  }

  fn process_event<T>(
    &mut self,
    event: GlobalHotKeyEvent,
    event_loop: &EventLoopWindowTarget<()>,
    manager: &Manager<T>,
  ) -> bool
  where
    T: Dispatchable,
  {
    macro_rules! exit {
      () => {
        // drop sender to stop processor
        drop(mem::take(&mut self.tx));
        event_loop.exit();
        return false;
      };
    }

    if event.id == exit_key().id() {
      log::info!("exiting gracefully after exit key pressed");
      exit!();
    }

    if event.state == HotKeyState::Pressed {
      if let Some(script) = manager.resolve(event.id) {
        if event.state == HotKeyState::Pressed
          && self
            .tx
            .as_ref()
            .expect("self.tx is only None at drop")
            .send(script)
            .is_err()
        {
          log::info!("exiting after Processor hung");
          exit!();
        }
      } else {
        log::error!("registered macro does not have corresponding script");
      }
    }

    true
  }
}

/// Put GlobalHotKeyManager in a valid state
fn hotkey_manager_register_keys(manager: &GlobalHotKeyManager, keys: &[HotKey]) {
  // register one by one, since some may already be registered...
  for key in keys {
    let _ = manager.register(*key);
  }
  // try to unregister exit key
  let _ = manager.unregister(exit_key());
  // re-register exit key for use by system
  let _ = manager.register(exit_key());
}
