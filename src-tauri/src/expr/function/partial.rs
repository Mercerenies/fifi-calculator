
//! Infrastructure for partial evaluation of functions.
//!
//! A full evaluation of a function is defined as a simplification
//! which takes a function call `f(x1, ..., xn)` to a single value,
//! assuming that the types of `x1, ..., xn` meet some conditions set
//! by the function.
//!
//! On the other hand, a partial evaluation of a function involves
//! taking `f(x1, ..., xn)` to `f(x1, ..., xi, N)`, where `N` is the
//! partial evaluation of `x(i+1), ..., xn`. That is, instead of
//! consuming all arguments and the function call, only some of the
//! arguments (i.e. those satisfying the function's type constraints)
//! are consumed.
//!
//! In order for a function to be eligible for partial evaluation, it
//! must, at minimum, satisfy
//! [`PERMITS_FLATTENING`](super::flags::PERMITS_FLATTENING). In that
//! case, continuous sequences of arguments which satisfy a function's
//! type constraints can be simplified. If the function also satisfies
//! [`PERMITS_REORDERING`](super::flags::PERMITS_REORDERING), then
//! arguments can be reordered to make sequences of simplifiable
//! arguments longer.

use crate::util::count_longest_subseq;

use itertools::Itertools;

/// The key type used to identify subsequences in
/// [`simplify_sequences`]. This type has a contrived (but correct)
/// `PartialEq` instance.
#[derive(Clone, Copy, Debug)]
enum SequenceKey {
  /// Key for values which do NOT satisfy the predicate. Note that
  /// `Unsatisfied` does NOT compare equal to itself.
  Unsatisfied,
  /// Key for values which do satisfy the predicate and should form a
  /// subsequence.
  Satisfied,
}

impl PartialEq for SequenceKey {
  fn eq(&self, other: &Self) -> bool {
    matches!((self, other), (SequenceKey::Satisfied, SequenceKey::Satisfied))
  }
}

impl From<bool> for SequenceKey {
  fn from(b: bool) -> Self {
    if b { SequenceKey::Satisfied } else { SequenceKey::Unsatisfied }
  }
}

/// Partial evaluator for functions which can be flattened
/// (generalized associativity) but do not necessarily admit
/// reordering (generalized commutativity).
///
/// Any consecutive sequence of two or more arguments which satisfy
/// `predicate` will be grouped together into the longest possible
/// subsequences and then passed to `evaluator`. `evaluator` should
/// return an `Ok(value)` on successful simplification, or the
/// original input argument list upon failure.
pub fn simplify_sequences<T, F, G>(
  args: Vec<T>,
  mut predicate: F,
  mut evaluator: G,
) -> Vec<T>
where F: FnMut(&T) -> bool,
      G: FnMut(Vec<T>) -> Result<T, Vec<T>> {
  // Fast track: Check if there's even anything to simplify. If not,
  // don't bother making a new vector. If there are no subsequences,
  // or all subsequences are of trivial length 1, then skip
  // simplifying.
  let longest_subseq_len = count_longest_subseq(&args, |x| predicate(x));
  if longest_subseq_len < 2 {
    return args;
  }

  let subsequences = args.into_iter().chunk_by(|item| SequenceKey::from(predicate(item)));
  let mut simplified_args = Vec::new();
  for (_key, subseq) in subsequences.into_iter() {
    let subseq: Vec<_> = subseq.collect();
    assert!(!subseq.is_empty(), "Expected nonempty subsequence from chunk_by");
    if subseq.len() == 1 {
      // Nothing to do; subsequence is trivial.
      simplified_args.extend(subseq);
    } else {
      let new_args = evaluator(subseq).map_or_else(|args| args, |arg| vec![arg]);
      simplified_args.extend(new_args);
    }
  }
  simplified_args
}

#[cfg(test)]
mod tests {
  use super::*;

  fn uncalled_evaluator(_: Vec<i64>) -> Result<i64, Vec<i64>> {
    panic!("Should not be called");
  }

  #[test]
  fn test_simplify_sequences_fast_track() {
    fn assert_eq_input(args: Vec<i64>, predicate: impl FnMut(&i64) -> bool) {
      assert_eq!(simplify_sequences(args.clone(), predicate, uncalled_evaluator), args);
    }

    assert_eq_input(vec![0, 1, 2, 3, 4], |_| false);
    assert_eq_input(vec![9], |_| true);
    assert_eq_input(vec![], |_| true);
    assert_eq_input(vec![0, 1, 2, 3, 4], |x| *x % 2 == 0);
    assert_eq_input(vec![1, 3, 3, 3, 3, 3, 1], |x| *x == 1);
    assert_eq_input(vec![1, 3, 3, 3, 1, 3, 3], |x| *x == 1);
  }

  #[test]
  fn test_simplify_sequences() {
    fn sum(args: Vec<i64>) -> Result<i64, Vec<i64>> {
      Ok(args.into_iter().sum())
    }

    assert_eq!(
      simplify_sequences(vec![0, 1, 2, 3, 4, 5, 6, 7], |_| true, sum),
      vec![28],
    );
    assert_eq!(
      simplify_sequences(vec![0, 1, 2, 3, 4, 5, 6, 7], |_| false, sum),
      vec![0, 1, 2, 3, 4, 5, 6, 7],
    );
    assert_eq!(
      simplify_sequences(vec![0, 1, 2, 3, 4, 5, 6, 7], |x| *x >= 3 && *x <= 6, sum),
      vec![0, 1, 2, 18, 7],
    );
    assert_eq!(
      simplify_sequences(vec![0, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 5, 1], |x| *x == 1, sum),
      vec![0, 4, 3, 5, 5, 1],
    );
  }
}
