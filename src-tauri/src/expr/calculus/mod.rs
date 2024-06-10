
//! Subsystems for doing basic calculus on expressions, such as taking
//! derivatives and integrals.

mod derivative;

pub use derivative::{DerivativeEngine, DifferentiationFailure, DifferentiationError, differentiate};
