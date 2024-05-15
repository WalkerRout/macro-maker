use std::{
  marker::PhantomData,
  mem,
  sync::mpsc::Receiver,
  thread::{self, JoinHandle},
};

use tokio::runtime::Builder;

use crate::command::command;

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
      }
    }
  }
}

#[derive(Debug, Default)]
pub struct Processor {
  rx: Option<Receiver<String>>,
}

impl Processor {
  pub fn new(rx: Receiver<String>) -> Self {
    Self { rx: Some(rx) }
  }

  pub fn spawn(&mut self) -> ProcessorGuard<'_> {
    let rx = mem::take(&mut self.rx);
    let handle = thread::spawn(|| {
      let rt = match Builder::new_multi_thread().build() {
        Ok(rt) => rt,
        _ => {
          log::error!("unable to construct new multi thread runtime");
          return;
        }
      };

      rt.block_on(async {
        if let Some(rx) = rx {
          loop {
            match rx.recv() {
              Ok(cmd_str) => {
                tokio::spawn(async move {
                  execute_command(cmd_str).await;
                });
              }
              _ => {
                log::warn!("Transmitter hung, stopping Processor for graceful exit");
                break;
              }
            }
          }
        }
      })
    });

    ProcessorGuard {
      handle: Some(handle),
      _phantom: PhantomData,
    }
  }
}

async fn execute_command(cmd_str: String) {
  let mut cmd = command(&cmd_str);
  // spawn and drop to detach child process
  match cmd.spawn() {
    Ok(_) => log::info!("command {cmd_str} successfully spawned"),
    Err(e) => log::error!("command {cmd_str} failed to spawn with {e}"),
  }
}
