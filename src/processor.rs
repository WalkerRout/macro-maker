use std::marker::PhantomData;
use std::mem;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

use async_process::Command;
use tokio::runtime::Builder;

use crate::Script;

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
  rx: Option<Receiver<Script>>,
}

impl Processor {
  pub fn with_receiver(rx: Receiver<Script>) -> Self {
    Self { rx: Some(rx) }
  }

  pub fn spin(&mut self) -> ProcessorGuard<'_> {
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
        let rx = rx.expect("self.rx should start with a valid state of Some(_)");
        loop {
          if let Ok(cmd_str) = rx.recv() {
            tokio::spawn(async move {
              execute_command(cmd_str.0).await;
            });
          } else {
            log::warn!("Transmitter hung, stopping Processor for graceful exit");
            break;
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

fn command<S: AsRef<str>>(cmd: S) -> Command {
  let tokens = command_tokens(cmd);
  if tokens.is_empty() {
    Command::new("")
  } else {
    let mut command = Command::new(&tokens[0]);
    command.args(&tokens[1..]);
    #[cfg(target_family = "windows")]
    {
      use async_process::windows::CommandExt;
      use winapi::um::winbase::CREATE_NO_WINDOW;
      command.creation_flags(CREATE_NO_WINDOW);
    }
    command
  }
}

fn command_tokens<S: AsRef<str>>(cmd: S) -> Vec<String> {
  let cmd = cmd.as_ref();

  let mut tokens = Vec::with_capacity(1);
  let mut string_buffer = String::new();

  let mut append_mode = false;
  let mut quote_mode = false;
  let mut quote_mode_ending = false; // to deal with '123''456' -> 123456
  let mut quote_char = ' ';
  let mut escaping = false;

  for c in cmd.chars() {
    if escaping {
      append_mode = true;
      escaping = false;
      string_buffer.push(c);
    } else if c.is_whitespace() {
      if append_mode {
        if quote_mode {
          string_buffer.push(c);
        } else {
          append_mode = false;
          tokens.push(string_buffer);
          string_buffer = String::new();
        }
      } else if quote_mode_ending {
        quote_mode_ending = false;
        tokens.push(string_buffer);
        string_buffer = String::new();
      }
    } else {
      match c {
        '"' | '\'' => {
          if append_mode {
            if quote_mode {
              if quote_char == c {
                append_mode = false;
                quote_mode = false;
                quote_mode_ending = true;
              } else {
                string_buffer.push(c);
              }
            } else {
              quote_mode = true;
              quote_char = c;
            }
          } else {
            append_mode = true;
            quote_mode = true;
            quote_char = c;
          }
        }
        '\\' => {
          escaping = true;
        }
        _ => {
          append_mode = true;
          escaping = false;
          string_buffer.push(c);
        }
      }
    }
  }

  if append_mode || quote_mode_ending {
    tokens.push(string_buffer);
  }

  tokens
}
