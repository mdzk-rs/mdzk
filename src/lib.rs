#[macro_use]
extern crate log;

pub mod build;
pub mod init;
pub mod preprocessors;
pub mod renderer;
pub mod serve;
pub mod utils;
pub mod watch;

pub use crate::{
    build::build,
    init::init, 
    serve::serve,
    renderer::HtmlMdzk,
};

pub const SRC_DIR: &str = "notes";
pub const BUILD_DIR: &str = "html";
pub const CONFIG_FILE: &str = "mdzk.toml";
pub const SUMMARY_FILE: &str = ".mdzk_summary.md";
pub const DEFAULT_ZK_TITLE: &str = "My mdzk";
