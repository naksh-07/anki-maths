// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! # Physics Domain Engine (Subsystem Layer)
//!
//! Provides Physics-specific physical model representations, dimensional compatibility,
//! unit systems, physical sanity validation, and kinematics / work-energy problem families.

pub mod diagnostics;
pub mod generators;
pub mod models;
pub mod sanity;
pub mod units;

pub use diagnostics::PhysicsErrorCategory;
pub use generators::*;
pub use models::{
    CoordinateSystem, PhysicalModelKind, PhysicalProblemMetadata, PhysicalQuantity, PhysicalRegime,
};
pub use sanity::PhysicalSanityValidator;
pub use units::{DimensionalValidator, PhysicalDimension, PhysicsUnit};
