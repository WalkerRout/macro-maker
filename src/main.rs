#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::LevelFilter;
use log4rs::{
  config::{self as cfg, Root, Appender},
  append::file::FileAppender,
  encode::pattern::PatternEncoder,
};

use dispatcher::Dispatcher;

mod command;
mod config;
mod dispatcher;

fn run() -> Result<(), anyhow::Error> {
  #[cfg(target_os = "macos")]
  {
    unimplemented!()
  }

  #[cfg(target_os = "linux")]
  {
    unimplemented!()
  }

  #[cfg(target_os = "windows")]
  {
    let mut dispatcher = Dispatcher::from_path("dispatch.toml")?;
    log::info!("STARTING");
    if let Err(e) = dispatcher.listen() {
      log::error!("exit with error: {e}");
    }
    log::info!("STOPPING\n");
  }

  Ok(())
}

fn main() -> Result<(), anyhow::Error> {
  let log_file = FileAppender::builder()
    .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} {l} - {m}{n}")))
    .build(format!("dispatch.log"))?;

  let config = cfg::Config::builder()
    .appender(Appender::builder().build("log", Box::new(log_file)))
    .build(Root::builder()
      .appender("log")
      .build(LevelFilter::Info))?;

  log4rs::init_config(config)?;

  run()
}
