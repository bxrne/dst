//! No-op component implementations for substrates that cannot virtualise a
//! component. All control operations inherit the trait defaults, which return
//! "not supported" errors. Kept for future substrates that don't support a
//! given component; the Docker substrate provides real implementations for
//! clock, network, and storage (`DockerStorage` via dm-flakey).

use super::clock::ClockControl;
use super::network::NetworkControl;
use super::storage::StorageControl;

/// No-op clock control for substrates without virtual clocks.
#[allow(dead_code)]
#[derive(Default)]
pub struct NopClock;

impl ClockControl for NopClock {}

/// No-op network control for substrates without network virtualisation.
#[allow(dead_code)]
#[derive(Default)]
pub struct NopNetwork;

impl NetworkControl for NopNetwork {}

/// No-op storage control for substrates without virtual storage.
#[allow(dead_code)]
#[derive(Default)]
pub struct NopStorage;

impl StorageControl for NopStorage {}
