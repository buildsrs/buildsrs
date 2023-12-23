use bytes::Bytes;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

#[cfg(feature = "frontend-vendor")]
static FRONTEND_FILES: &[(&str, &[u8])] = &include!(concat!(env!("OUT_DIR"), "/frontend_files.rs"));

/// Files
pub type Files = BTreeMap<PathBuf, Bytes>;

/// Read-only, shared files
pub type SharedFiles = Arc<Files>;

/// Files for the frontend
#[cfg(feature = "frontend-vendor")]
pub fn frontend() -> Files {
    let mut files = Files::default();

    for (name, data) in FRONTEND_FILES {
        tracing::debug!("Loading {name} as frontend file");
        files.insert(PathBuf::from(name), Bytes::from(*data));
    }

    files
}
