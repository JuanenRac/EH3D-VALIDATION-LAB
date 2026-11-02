// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-VALIDATION-LAB: the test environment of the PHOENIX family.
//!
//! This version holds the parts that need no design model: manifests of golden sets with
//! redundant copies, failures that must carry a class, and the gate that a measured rate never
//! drops. Nothing here reads a design file, and nothing runs a reference tool.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod classify;
pub mod cli;
pub mod config;
pub mod error;
pub mod gate;
pub mod hash;
pub mod manifest;
pub mod verify;

pub use error::LabError;
