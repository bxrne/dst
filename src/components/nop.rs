//! No-op component implementations for substrates that cannot virtualise a
//! component. All control operations inherit the trait defaults, which return
//! "not supported" errors; `ClockControl::now` still reports real wall time.

use super::network::NetworkControl;
use super::storage::StorageControl;

#[derive(Default)]
pub struct NopNetwork;

impl NetworkControl for NopNetwork {}

#[derive(Default)]
pub struct NopStorage;

impl StorageControl for NopStorage {}
