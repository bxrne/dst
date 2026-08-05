//! No-op component implementations for substrates that cannot virtualise a
//! component. All control operations inherit the trait defaults, which return
//! "not supported" errors. Kept for future substrates that don't support a
//! given component; the Docker substrate provides real implementations for
//! clock and network, and `NopStorage` for storage (until dm-flakey lands).

use super::network::NetworkControl;
use super::storage::StorageControl;

/// No-op network control for substrates without network virtualisation.
#[allow(dead_code)]
#[derive(Default)]
pub struct NopNetwork;

impl NetworkControl for NopNetwork {}

/// No-op storage control for substrates without virtual storage.
#[derive(Default)]
pub struct NopStorage;

impl StorageControl for NopStorage {}
