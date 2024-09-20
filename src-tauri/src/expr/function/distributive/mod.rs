
mod eval;
mod rule;

pub use eval::{DistributivePropertyError, DistributivePropertyErrorDetails, distribute_over};
pub use rule::{DistributiveRule, DistributiveArgRule, DistributiveRuleNotApplicable, Side};
