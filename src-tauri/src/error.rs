
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
  #[error("{0}")]
  TauriError(#[from] tauri::Error),
}
