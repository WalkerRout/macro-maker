use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
use global_hotkey::hotkey::{Code, HotKey, Modifiers as Mods};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;

use crate::{Dispatchable, Script}; // DeserializeOwned

pub struct Monitor {
  _watcher: RecommendedWatcher,
}

impl Monitor {
  fn watch<T, P>(
    path: P,
    config: Arc<Mutex<T>>,
    update_tx: Sender<()>,
  ) -> Result<Self, anyhow::Error>
  where
    P: AsRef<Path>,
    T: for<'a> Deserialize<'a> + Dispatchable + Send + 'static,
  {
    let opath = path.as_ref().to_path_buf();
    let mut _watcher = notify::recommended_watcher(move |res| {
      match res {
        Ok(Event {
          kind: EventKind::Modify(_),
          ..
        }) => {
          if let Ok(toml) = fs::read_to_string(opath.clone()) {
            if let Ok(new_config) = toml::from_str::<T>(&toml) {
              // successful interpretation of config - update it!
              *config.lock().unwrap() = new_config;
              let _ = update_tx.send(());
            }
          }
        }
        Err(e) => log::error!("watch error: {e}"),
        _ => (),
      }
    })?;
    _watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
    Ok(Self { _watcher })
  }
}

pub struct Manager<T> {
  config: Arc<Mutex<T>>,
  hotkey_update_rx: Receiver<()>,
  _monitor: Monitor,
}

impl<T> Manager<T> {
  pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error>
  where
    T: for<'a> Deserialize<'a> + Send + Default + Dispatchable + 'static,
  {
    let dispatch_toml = fs::read_to_string(&path)?;
    // config can be in an unstable state in the beginning; it will start with 0 keybinds
    let config = toml::from_str::<T>(&dispatch_toml).unwrap_or_default();
    let config = Arc::new(Mutex::new(config));
    let (update_tx, update_rx) = bounded(1);
    Ok(Self {
      config: Arc::clone(&config),
      hotkey_update_rx: update_rx,
      _monitor: Monitor::watch(path, config, update_tx)?,
    })
  }

  pub fn resolve(&self, id: u32) -> Option<Script>
  where
    T: Dispatchable,
  {
    self.config.lock().unwrap().scriptify(id)
  }

  pub fn hotkeys(&self) -> Vec<HotKey>
  where
    T: Dispatchable,
  {
    self.config.lock().unwrap().hotkeys()
  }

  pub fn try_update(&self) -> Result<(), TryRecvError> {
    self.hotkey_update_rx.try_recv()
  }
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
  pub commands: Vec<Command>,
}

impl Dispatchable for Config {
  fn hotkeys(&self) -> Vec<HotKey> {
    self.commands.iter().map(Command::as_hotkey).collect()
  }

  fn scriptify(&self, id: u32) -> Option<Script> {
    self.commands.iter().find_map(|command| {
      if command.as_hotkey().id() == id {
        Some(Script(command.script.clone()))
      } else {
        None
      }
    })
  }
}

// optional modifiers let users omit unneeded keys
#[derive(Debug, Default, Deserialize)]
pub struct Command {
  modifiers: Modifiers,
  hotkey: String,
  script: String,
}

impl Command {
  fn as_hotkey(&self) -> HotKey {
    HotKey::new(Some(self.modifiers()), self.key_code())
  }

  fn modifiers(&self) -> Mods {
    let def = Mods::empty();
    let get = |b, mod_type| if b { mod_type } else { def };
    #[rustfmt::skip]
    let mods = self.modifiers.alt.map_or(def, |b| get(b, Mods::ALT))
      | self.modifiers.meta.map_or(def, |b| get(b, Mods::META))
      | self.modifiers.shift.map_or(def, |b| get(b, Mods::SHIFT))
      | self.modifiers.control.map_or(def, |b| get(b, Mods::CONTROL));
    mods
  }

  fn key_code(&self) -> Code {
    use std::str::FromStr;
    Code::from_str(&self.hotkey).unwrap()
  }
}

#[derive(Debug, Default, Deserialize)]
struct Modifiers {
  alt: Option<bool>,  // option
  meta: Option<bool>, // win, super, cmd
  shift: Option<bool>,
  control: Option<bool>,
}

pub fn exit_key() -> HotKey {
  let key_code = Code::KeyE;
  let modifiers = Mods::SHIFT | Mods::CONTROL | Mods::ALT;
  HotKey::new(Some(modifiers), key_code)
}
