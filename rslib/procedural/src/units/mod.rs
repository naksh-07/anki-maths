// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! # Unified Physical & Chemical Dimensional Analysis and Unit Conversion Engine
//!
//! Provides deterministic 5-dimensional vector analysis ($[M]^a [L]^b [T]^c [N]^d [K]^e$),
//! parsing for equations, prefixes, negative numbers, fractions, scientific notation,
//! and accurate cross-unit equivalence verification across Physics, Chemistry, and Math.

pub mod dimension;
pub mod parser;
pub mod quantity;
pub mod tolerance;
pub mod unit_def;
pub mod validator;

pub use dimension::Dimension;
pub use parser::{ParsedQuantity, UnitParser};
pub use quantity::Quantity;
pub use tolerance::Tolerance;
pub use unit_def::Unit;
pub use validator::UnitAnswerValidator;
