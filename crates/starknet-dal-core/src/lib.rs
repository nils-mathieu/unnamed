//! Provides a generic framework for building custom data availability layers for Starknet.
//!
//! There's two types of users for this crate:
//!
//! 1. Users who want their software to remain generic over the data availability layer they use
//! (e.g. ethereum, etc).
//!
//! 2. Users who want to build a custom data availability layer for Starknet that 1. can use
//!    in their own projects with no added modifications.

/// Represents a data availability layer responsible for providing data to the Starknet
/// network without relying (necessarily) on a local database.
pub trait DataAvailabilityLayer {}
