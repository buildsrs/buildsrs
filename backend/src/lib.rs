use anyhow::Result;
use clap::Parser;

mod api;
mod bucket;
mod options;
mod state;

pub use crate::{options::Options, state::Backend};
