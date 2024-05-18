use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::thread::JoinHandle;

use global_hotkey::hotkey::HotKey;

use manager::Manager;

pub use std::sync::mpsc::Receiver;
pub use std::sync::mpsc::Sender;

pub mod dispatcher;
pub mod manager;
pub mod processor;
pub mod transmitter;
// pub mod hotkey; // TODO: implement wrapper for generic hotkey backend

pub trait Dispatch {
  fn hotkeys(&self) -> Vec<HotKey>;
  fn scriptify(&self, id: u32) -> Option<Script>;
}

pub trait Transmit {
  fn with_sender(tx: Sender<Script>) -> Self;
  fn listen_for_hotkeys<T>(&mut self, manager: &Manager<T>)
  where
    T: Dispatch;
}

pub trait Process {
  fn with_receiver(rx: Receiver<Script>) -> Self;
  fn process_incoming_scripts(&mut self) -> ProcessorGuard<'_>;
}

#[derive(Debug)]
pub struct ProcessorGuard<'a> {
  handle: Option<JoinHandle<()>>,
  _phantom: PhantomData<&'a ()>,
}

impl<'a> Drop for ProcessorGuard<'a> {
  fn drop(&mut self) {
    if let Some(handle) = mem::take(&mut self.handle) {
      if let Err(e) = handle.join() {
        log::error!("failed to join Processor with {e:?}");
      } else {
        log::info!("Processor terminated")
      }
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct Script(pub String);

impl Deref for Script {
  type Target = String;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
