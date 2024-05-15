use std::sync::mpsc::Sender;

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use winit::event_loop::{ControlFlow, EventLoopBuilder, EventLoopWindowTarget};

use crate::config::{exit_key, KeyIdMap};

#[derive(Debug)]
pub struct Transmitter {
  tx: Sender<String>,
}

impl Transmitter {
  pub fn new(tx: Sender<String>) -> Self {
    Self { tx }
  }

  pub fn spin(&mut self, key_map: KeyIdMap) {
    let event_loop = EventLoopBuilder::new()
      .build()
      .expect("event loop spun twice");
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    event_loop
      .run(move |_event, event_loop| {
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Ok(event) = global_hotkey_channel.try_recv() {
          self.process_event(event, event_loop, &key_map);
        }
      })
      .expect("run event loop");
  }

  fn process_event(
    &mut self,
    event: GlobalHotKeyEvent,
    event_loop: &EventLoopWindowTarget<()>,
    key_map: &KeyIdMap,
  ) {
    // exit
    if event.id == exit_key().id() {
      log::info!("exiting gracefully after exit key pressed");
      event_loop.exit();
      return;
    }
    // TODO: check if things changed in file...
    // - if they did, reload self with the dispatch path!
    if let Some(command) = key_map.get(&event.id) {
      if event.state == HotKeyState::Pressed {
        if let Err(cmd) = self.tx.send(command.clone()) {
          log::info!("exiting unsuccessfully after trying to send {cmd} to dropped Processor");
          event_loop.exit();
        }
      }
    } else {
      log::error!("registered macro does not have corresponding script");
    }
  }
}
