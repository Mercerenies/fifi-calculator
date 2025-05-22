
//! The [`PrismTupleList`] trait and associated [`narrow_vec`] helper
//! function. See the specific documentation on those names for
//! details.

use crate::util::prism::Prism;

use tuple_list::{Tuple, TupleList};

/// Assuming `Xs` is a tuple list (as defined by the [`tuple_list`]
/// crate), a `PrismTupleList<A, Xs>` is a tuple list of prisms (more
/// precisely, a tuple list of references to prisms), where each prism
/// takes the single type `A` down to the corresponding element in
/// `Xs`.
///
/// This trait provides the
/// [`narrow_iter`](PrismTupleList::narrow_iter) function, which
/// narrows an arbitrary iterable of `A` values into the tuple list
/// `Xs`. In case of failure for any reason during narrowing, the
/// full, original iterable is recovered (via [`Prism::widen_type`]
/// where necessary) and returned as a vector.
///
/// Users should not implement this trait on their own. It is
/// automatically implemented for all tuple lists which satisfy the
/// criterion.
///
/// See also: [`narrow_vec`], which operates on tuples rather than
/// tuple lists for ergonomic reasons and is also capable of
/// short-circuiting out in case of a non-matching vector length.
pub trait PrismTupleList<A, Xs> {
  /// Narrows the iterable to a tuple list of results, according to
  /// the prisms contained in `self`. See the trait-level
  /// documentation for more details.
  fn narrow_iter<I>(&self, iter: I) -> Result<Xs, Vec<A>>
  where I: IntoIterator<Item=A>;

  // TODO widen_iter, if we need it :)
}

impl<A> PrismTupleList<A, ()> for () {
  fn narrow_iter<I>(&self, iter: I) -> Result<(), Vec<A>>
  where I: IntoIterator<Item=A> {
    let mut iter = iter.into_iter();
    if let Some(next) = iter.next() {
      // We expected the iterator to be empty. Failure.
      Err(iter_rebuild(next, iter))
    } else {
      Ok(())
    }
  }
}

impl<A, X, Xs, P, Ps> PrismTupleList<A, (X, Xs)> for (&P, Ps)
where P: Prism<A, X>,
      Ps: PrismTupleList<A, Xs> {
  fn narrow_iter<I>(&self, iter: I) -> Result<(X, Xs), Vec<A>>
  where I: IntoIterator<Item=A> {
    let mut iter = iter.into_iter();
    let Some(next) = iter.next() else {
      return Err(Vec::new());
    };
    match self.0.narrow_type(next) {
      Ok(next) => {
        match self.1.narrow_iter(iter) {
          Ok(tail) => {
            Ok((next, tail))
          }
          Err(mut tail) => {
            tail.insert(0, self.0.widen_type(next));
            Err(tail)
          }
        }
      }
      Err(next) => {
        Err(iter_rebuild(next, iter))
      }
    }
  }
}

/// Narrows a homogeneous vector of values, using a tuple of prisms,
/// to get a tuple of results. If the input vector has the wrong
/// length or any of the prisms fails, the original vector is
/// recovered and returned.
///
/// This function is implemented in terms of
/// [`PrismTupleList::narrow_iter`], but has two benefits over that
/// trait method. The first is that this function takes and returns
/// tuples, which are often more ergonomic to work with than the
/// alternative of tuple lists. The second is that, since this
/// function is specialized to take a vector, it can check the length
/// in advance and bail out early if there's no possibility of
/// successfully matching due to a length error.
///
/// Assuming lawful prisms, this function will either succeed or
/// return a vector indistinguishable from the input one.
pub fn narrow_vec<A, Xs, Ps>(prisms: Ps, vec: Vec<A>) -> Result<Xs::Tuple, Vec<A>>
where Ps: Tuple,
      Ps::TupleList: PrismTupleList<A, Xs>,
      Xs: TupleList {
  if vec.len() != Ps::TupleList::TUPLE_LIST_SIZE {
    return Err(vec);
  }

  let prisms = prisms.into_tuple_list();
  let narrowed_values = prisms.narrow_iter(vec)?;
  Ok(narrowed_values.into_tuple())
}

/// Takes a value and an iterator over the same type. Produces a
/// vector whose first element is the value and whose tail is the
/// remaining items in the iterator.
///
/// Example:
/// ```
/// let v = vec![20, 30, 40];
/// assert_eq!(iter_rebuild(10, v.iter()), vec![10, 20, 30, 40]);
/// ```
fn iter_rebuild<I: Iterator>(curr: I::Item, iter: I) -> Vec<I::Item> {
  let mut values = vec![curr];
  values.extend(iter);
  values
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::util::prism::Identity;

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  struct Pos(i64);

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  struct Neg(i64);

  /// Test prism which asserts that its argument is strictly positive.
  struct I64ToPos;

  /// Test prism which asserts that its argument is strictly negative.
  struct I64ToNeg;

  impl Prism<i64, Pos> for I64ToPos {
    fn narrow_type(&self, i: i64) -> Result<Pos, i64> {
      if i <= 0 {
        Err(i)
      } else {
        Ok(Pos(i))
      }
    }

    fn widen_type(&self, pos: Pos) -> i64 {
      pos.0
    }
  }

  impl Prism<i64, Neg> for I64ToNeg {
    fn narrow_type(&self, i: i64) -> Result<Neg, i64> {
      if i >= 0 {
        Err(i)
      } else {
        Ok(Neg(i))
      }
    }

    fn widen_type(&self, neg: Neg) -> i64 {
      neg.0
    }
  }

  #[test]
  fn test_iter_rebuild() {
    let v = vec![20, 30, 40];
    assert_eq!(iter_rebuild(10, v.into_iter()), vec![10, 20, 30, 40]);
  }

  #[test]
  fn test_iter_rebuild_on_empty_iter() {
    let v = vec![];
    assert_eq!(iter_rebuild(10, v.into_iter()), vec![10]);
  }

  #[test]
  fn test_narrow_iter_on_unit() {
    let prisms = ();
    assert_eq!(prisms.narrow_iter(Vec::<i64>::new()), Ok(()));
    assert_eq!(prisms.narrow_iter(vec![10]), Err(vec![10]));
    assert_eq!(prisms.narrow_iter(vec![10, 20]), Err(vec![10, 20]));
  }

  #[test]
  fn test_narrow_iter_on_nontrivial_prisms() {
    let prisms = (&I64ToPos, (&Identity, (&I64ToNeg, ())));
    assert_eq!(prisms.narrow_iter(Vec::<i64>::new()), Err(Vec::<i64>::new()));
    assert_eq!(
      prisms.narrow_iter(vec![10, 20, -30]),
      Ok((Pos(10), (20, (Neg(-30), ())))),
    );
    assert_eq!(
      prisms.narrow_iter(vec![10, 20, 30]),
      Err(vec![10, 20, 30]),
    );
    assert_eq!(
      prisms.narrow_iter(vec![10, 20, -30, 0, 1, 2]),
      Err(vec![10, 20, -30, 0, 1, 2]),
    );
  }

  #[test]
  fn test_narrow_vec_helper_function() {
    let prisms = (&I64ToPos, &Identity, &I64ToNeg);
    assert_eq!(narrow_vec(prisms.clone(), Vec::<i64>::new()), Err(Vec::<i64>::new()));
    assert_eq!(
      narrow_vec(prisms.clone(), vec![10, 20, -30]),
      Ok((Pos(10), 20, Neg(-30))),
    );
    assert_eq!(
      narrow_vec(prisms.clone(), vec![10, 20, 30]),
      Err(vec![10, 20, 30]),
    );
    assert_eq!(
      narrow_vec(prisms, vec![10, 20, -30, 0, 1, 2]),
      Err(vec![10, 20, -30, 0, 1, 2]),
    );
  }
}
