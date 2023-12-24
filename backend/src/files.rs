use bytes::Bytes;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

/// Mapping of file path to content
pub type Files = BTreeMap<PathBuf, Bytes>;

/// Read-only, shared files
pub type SharedFiles = Arc<Files>;

#[cfg(feature = "frontend-vendor")]
mod vendor {
    use super::{Bytes, Files, PathBuf};

    static FRONTEND_FILES: &[(&str, &[u8])] =
        &include!(concat!(env!("OUT_DIR"), "/frontend_files.rs"));

    #[test]
    fn has_index_html() {
        assert!(FRONTEND_FILES
            .into_iter()
            .any(|(name, _content)| name == &"index.html"));
    }

    /// Files for the frontend
    pub fn frontend() -> Files {
        let mut files = Files::default();

        for (name, data) in FRONTEND_FILES {
            tracing::debug!("Loading {name} as frontend file");
            files.insert(PathBuf::from(name), Bytes::from(*data));
        }

        files
    }

    #[test]
    fn can_build_frontend_files() {
        let files = frontend();
        assert!(files.len() > 0);
    }
}

#[cfg(feature = "frontend-vendor")]
pub use vendor::frontend;
