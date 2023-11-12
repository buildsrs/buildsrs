//! # Buildsrs Backend
//!
//! This crate implements the backend of the buildsrs project. It exposes a REST API that allows
//! for fetching crate metadata, list and download artifacts. In addition, it also exposes a
//! WebSocket that the builders connect to in order to fetch build jobs and stream logs.
//!
//! Persistence is not implemented here, but abstracted away by the
//! [`Storage`](buildsrs_storage::Storage) and [`Database`](buildsrs_database::Database) types and
//! traits.

#![warn(missing_docs)]

mod api;
mod state;

pub use crate::state::Backend;
