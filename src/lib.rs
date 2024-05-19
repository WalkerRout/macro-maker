use std::ops::Deref;

use global_hotkey::hotkey::HotKey;

pub use std::sync::mpsc::Receiver;
pub use std::sync::mpsc::Sender;

pub mod dispatcher;
pub mod manager;
pub mod process;
pub mod transmit;
// pub mod hotkey; // TODO: implement wrapper for generic hotkey backend

pub trait Dispatch {
  fn hotkeys(&self) -> Vec<HotKey>;
  fn scriptify(&self, id: u32) -> Option<Script>;
}

#[derive(Debug, Default, Clone)]
pub struct Script(pub String);

impl Deref for Script {
  type Target = String;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
