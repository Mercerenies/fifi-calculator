
use std::collections::HashMap;

pub struct MetricPrefix {
  pub prefix_name: String,
  pub exponent: i64,
}

impl MetricPrefix {
  pub fn new(prefix_name: impl Into<String>, exponent: i64) -> MetricPrefix {
    MetricPrefix {
      prefix_name: prefix_name.into(),
      exponent,
    }
  }

  pub fn si_prefixes() -> Vec<MetricPrefix> {
    vec![
      MetricPrefix::new("Q", 30),
      MetricPrefix::new("R", 27),
      MetricPrefix::new("Y", 24),
      MetricPrefix::new("Z", 21),
      MetricPrefix::new("E", 18),
      MetricPrefix::new("P", 15),
      MetricPrefix::new("T", 12),
      MetricPrefix::new("G", 9),
      MetricPrefix::new("M", 6),
      MetricPrefix::new("k", 3),
      MetricPrefix::new("h", 2),
      MetricPrefix::new("D", 1),
      MetricPrefix::new("d", -1),
      MetricPrefix::new("c", -2),
      MetricPrefix::new("m", -3),
      // Note: We accept both "u" and "μ" for micro.
      MetricPrefix::new("u", -6),
      MetricPrefix::new("μ", -6),
      MetricPrefix::new("n", -9),
      MetricPrefix::new("p", -12),
      MetricPrefix::new("f", -15),
      MetricPrefix::new("a", -18),
      MetricPrefix::new("z", -21),
      MetricPrefix::new("y", -24),
      MetricPrefix::new("r", -27),
      MetricPrefix::new("q", -30),
    ]
  }

  pub fn si_prefixes_map() -> HashMap<String, MetricPrefix> {
    Self::si_prefixes()
      .into_iter()
      .map(|prefix| (prefix.prefix_name.clone(), prefix))
      .collect()
  }
}
