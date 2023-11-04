use anyhow::Result;
use clap::Parser;

mod api;
mod bucket;
mod options;
mod state;
mod storage;

pub use crate::{options::Options, state::Backend};
