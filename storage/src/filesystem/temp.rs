use super::*;
use tempfile::TempDir;

impl Filesystem {
    /// Create a temporary filesystem storage.
    pub async fn new_temp() -> Temporary<Filesystem> {
        let dir = TempDir::new().unwrap();
        let storage = Filesystem::new(dir.path().to_path_buf());
        let cleanup = async move {
            dir.close().unwrap();
        };
        Temporary::new(storage, Box::pin(cleanup))
    }
}
