#[macro_use]
extern crate log;

pub mod cmd;
pub mod config;
pub mod renderer;
pub mod utils;
pub mod zk;

#[doc(inline)]
pub use crate::{cmd::build, cmd::init, cmd::serve, config::Config, zk::load_zk};

pub const SRC_DIR: &str = "notes";
pub const BUILD_DIR: &str = "html";
pub const CONFIG_FILE: &str = "mdzk.toml";
pub const SUMMARY_FILE: &str = ".mdzk_summary.md";
pub const DEFAULT_ZK_TITLE: &str = "My mdzk";
