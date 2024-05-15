use std::{
  collections::HashMap,
  fs,
  path::Path,
  sync::mpsc::{self, Receiver, Sender},
  thread,
};

use global_hotkey::{
  hotkey::{Code, HotKey, Modifiers as Mods},
  GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use tokio::runtime::Builder;
use winit::event_loop::{ControlFlow, EventLoopBuilder};

use crate::command::command;
use crate::config::Config;

pub struct Dispatcher {
  key_map: HashMap<u32, String>,
  _manager: GlobalHotKeyManager,
}

impl Dispatcher {
  pub fn new(key_map: HashMap<u32, String>, _manager: GlobalHotKeyManager) -> Self {
    // remove exit key if accidentally registered by user
    if _manager.unregister(exit_key()).is_ok() {
      log::warn!("do not register Ctrl+Shift+Alt+KeyE; it is the built in exit key");
    }
    // re-register exit key for use by system
    let _ = _manager.register(exit_key());
    Self { key_map, _manager }
  }

  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
    let dispatch_toml = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&dispatch_toml)?;
    let key_map: HashMap<HotKey, String> = config.into();
    let manager = GlobalHotKeyManager::new()?;
    for key in key_map.keys() {
      let _ = manager.register(*key);
    }
    let key_map = {
      let mut map = HashMap::with_capacity(key_map.len());
      for (hotkey, value) in key_map {
        map.insert(hotkey.id(), value);
      }
      map
    };
    Ok(Self::new(key_map, manager))
  }

  pub fn listen(&mut self) -> Result<(), anyhow::Error> {
    let (tx, rx) = mpsc::channel::<String>();
    let handle = thread::spawn(|| Self::dispatch_processes(rx));
    Self::listen_for_macros(self, tx)?;
    handle.join().map_err(|e| anyhow::anyhow!("{e:?}"))
  }

  fn dispatch_processes(rx: Receiver<String>) {
    let rt = match Builder::new_multi_thread().build() {
      Ok(rt) => rt,
      _ => {
        log::error!("unable to construct new multi thread runtime");
        panic!();
      }
    };

    rt.block_on(async {
      loop {
        match rx.recv() {
          Ok(cmd_str) => {
            tokio::spawn(async move {
              log::info!("running command {}", cmd_str.as_str());
              let mut cmd = command(cmd_str.as_str());
              match cmd.output().await {
                Ok(out) => log::info!("command {} succeeded with {}", cmd_str, out.status),
                Err(e) => log::error!("command {} failed with {e}", cmd_str),
              }
            });
          }
          _ => {
            log::warn!("sender hung, dropping processes for graceful exit");
            break;
          }
        }
      }
    })
  }

  fn listen_for_macros(&mut self, tx: Sender<String>) -> Result<(), anyhow::Error> {
    let event_loop = EventLoopBuilder::new().build()?;
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    event_loop
      .run(move |_event, event_loop| {
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Ok(event) = global_hotkey_channel.try_recv() {
          // exit
          if event.id == exit_key().id() {
            log::info!("exiting gracefully after exit key pressed");
            event_loop.exit();
            return;
          }
          // check if things changed in file...
          // - if they did, reload self with the dispatch path!
          if let Some(command) = self.key_map.get(&event.id) {
            if event.state == HotKeyState::Pressed {
              let _ = tx.send(command.clone());
            }
          } else {
            log::error!("registered macro does not have corresponding script");
          }
        }
      })
      .unwrap();
    Ok(())
  }
}

fn exit_key() -> HotKey {
  let key_code = Code::KeyE;
  let modifiers = Mods::SHIFT | Mods::CONTROL | Mods::ALT;
  HotKey::new(Some(modifiers), key_code)
}
