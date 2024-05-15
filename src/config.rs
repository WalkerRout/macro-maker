use std::{collections::HashMap, fs, mem, ops::Deref, path::Path, str::FromStr};

use global_hotkey::hotkey::{Code, HotKey, Modifiers as Mods};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
struct Modifiers {
  alt: Option<bool>,  // option
  meta: Option<bool>, // win, super, cmd
  shift: Option<bool>,
  control: Option<bool>,
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
    let get = move |b, that| if b { that } else { def };

    let alt = self.modifiers.alt.map(|b| get(b, Mods::ALT)).unwrap_or(def);
    let meta = self
      .modifiers
      .meta
      .map(|b| get(b, Mods::META))
      .unwrap_or(def);
    let shift = self
      .modifiers
      .shift
      .map(|b| get(b, Mods::SHIFT))
      .unwrap_or(def);
    let control = self
      .modifiers
      .control
      .map(|b| get(b, Mods::CONTROL))
      .unwrap_or(def);

    alt | meta | shift | control
  }

  fn key_code(&self) -> Code {
    Code::from_str(&self.hotkey).unwrap()
  }
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
  pub commands: Vec<Command>,
}

impl From<Config> for HashMap<HotKey, String> {
  fn from(config: Config) -> Self {
    let keys = config
      .commands
      .iter()
      .map(|c| c.as_hotkey())
      .collect::<Vec<_>>();
    let scripts = config
      .commands
      .into_iter()
      .map(|c| c.script)
      .collect::<Vec<_>>();
    keys.into_iter().zip(scripts).collect()
  }
}

#[derive(Debug, Default)]
pub struct KeyMap(HashMap<HotKey, String>);

impl KeyMap {
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
    let dispatch_toml = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&dispatch_toml)?;
    Ok(Self(config.into()))
  }
}

impl Deref for KeyMap {
  type Target = HashMap<HotKey, String>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Debug, Default)]
pub struct KeyIdMap(HashMap<u32, String>);

impl From<KeyMap> for KeyIdMap {
  fn from(mut key_map: KeyMap) -> Self {
    let mut map = HashMap::with_capacity(key_map.len());
    for (hotkey, value) in mem::take(&mut key_map.0) {
      map.insert(hotkey.id(), value);
    }
    Self(map)
  }
}

impl Deref for KeyIdMap {
  type Target = HashMap<u32, String>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub fn exit_key() -> HotKey {
  let key_code = Code::KeyE;
  let modifiers = Mods::SHIFT | Mods::CONTROL | Mods::ALT;
  HotKey::new(Some(modifiers), key_code)
}
