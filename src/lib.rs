//! loam-core — the demand side of Loam (ADR-0006).
//!
//! Reads an OKF bundle (canonical markdown), the derived index, the inbox, and
//! the telemetry spool. Never writes concepts (invariant: agents-never-write).
//! Built ahead of the P0 gate against a fixture bundle per ADR-0007.

pub mod bundle;
pub mod concept;
pub mod get;
pub mod search;
pub mod spool;
