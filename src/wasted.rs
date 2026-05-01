//! Wasted/one-off utilities.
//!
//! This module contains one-off utilities that don't fit elsewhere.

pub mod aery_fix;

/// Fix Aery folders by merging similar directories.
pub use aery_fix::aery_fix;
