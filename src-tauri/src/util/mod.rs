
//! Various utility functions.

pub mod angles;
pub mod cow_dyn;
pub mod matrix;
pub mod point;
pub mod prism;
pub mod radix;
pub mod stricteq;

use regex::{Regex, escape};
use either::Either;
use num::One;
use num::pow::Pow;

use std::fmt::{self, Formatter, Display};
use std::convert::Infallible;
use std::cmp::{Reverse, Ordering};
use std::iter::{self, Extend};
use std::ops::{Mul, Neg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sign {
  Negative,
  Positive,
}

pub trait Recip {
  type Output;

  fn recip(self) -> Self::Output;
}

/// `try_traits`-style variant of [`Pow`] trait.
pub trait TryPow<RHS> {
  type Output;
  type Error;

  fn try_pow(self, rhs: RHS) -> Result<Self::Output, Self::Error>;
}

/// [`One`] without the [`Mul`] constraint.
pub trait PreOne: Sized {
  fn pre_one() -> Self;
  fn is_pre_one(&self) -> bool;
}

impl Sign {
  pub fn other(self) -> Self {
    match self {
      Self::Negative => Self::Positive,
      Self::Positive => Self::Negative,
    }
  }
}

impl Recip for f32 {
  type Output = f32;

  fn recip(self) -> Self::Output {
    self.recip()
  }
}

// Note: The PartialEq bound here will hopefully be removable once
// num::One removes that bound on their end.
impl<T: One + PartialEq> PreOne for T {
  fn pre_one() -> Self {
    T::one()
  }

  fn is_pre_one(&self) -> bool {
    self.is_one()
  }
}

impl<T, RHS> TryPow<RHS> for T
where T: Pow<RHS> {
  type Output = T::Output;
  type Error = Infallible;

  fn try_pow(self, rhs: RHS) -> Result<Self::Output, Self::Error> {
    Ok(self.pow(rhs))
  }
}

impl Recip for f64 {
  type Output = f64;

  fn recip(self) -> Self::Output {
    self.recip()
  }
}

impl Display for Sign {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Negative => write!(f, "-"),
      Self::Positive => write!(f, "+"),
    }
  }
}

impl Mul for Sign {
  type Output = Self;

  fn mul(self, other: Self) -> Self::Output {
    if self == other {
      Self::Positive
    } else {
      Self::Negative
    }
  }
}

impl Neg for Sign {
  type Output = Self;

  fn neg(self) -> Self::Output {
    self.other()
  }
}

pub fn unwrap_infallible<T>(res: Result<T, Infallible>) -> T {
  match res {
    Ok(res) => res,
    Err(_) => unreachable!(),
  }
}

/// Constructs a regex which matches any string in `options`.
pub fn regex_opt<'a, I>(options: I) -> Regex
where I : IntoIterator<Item = &'a str> {
  regex_opt_with(options, |s| s)
}

/// Constructs a regex which matches any string in `options`. Applies
/// the function `helper` to the resulting regex string before
/// compilation. If the result of `helper` is not a valid regular
/// expression, this function will panic.
pub fn regex_opt_with<'a, I, F>(options: I, helper: F) -> Regex
where I : IntoIterator<Item = &'a str>,
      F : FnOnce(String) -> String {
  // Put longer elements first, so we always match the longest thing
  // we can.
  let mut options: Vec<_> = options.into_iter().collect();
  options.sort_by_key(|a| Reverse(a.len()));

  let regex_str = options.into_iter().map(escape).collect::<Vec<_>>().join("|");
  let regex_str = helper(format!("(?:{regex_str})"));
  Regex::new(&regex_str).unwrap_or_else(|_| {
    panic!("Invalid regular expression: {}", regex_str);
  })
}

pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
  if val < min { min } else if val > max { max } else { val }
}

/// Returns a count of the number of elements at the beginning of this
/// iterator which satisfy the predicate.
pub fn count_prefix<I, F>(iter: I, mut predicate: F) -> usize
where F: FnMut(I::Item) -> bool,
      I: Iterator {
  let mut count = 0;
  for item in iter {
    if predicate(item) {
      count += 1;
    } else {
      break;
    }
  }
  count
}

/// Returns a count of the number of elements at the end of this
/// (double-ended) iterator which satisfy the predicate.
pub fn count_suffix<I, F>(iter: I, mut predicate: F) -> usize
where F: FnMut(I::Item) -> bool,
      I: Iterator + DoubleEndedIterator {
  let mut count = 0;
  for item in iter.rev() {
    if predicate(item) {
      count += 1;
    } else {
      break;
    }
  }
  count
}

/// Produces a container which consists of the same element repeated
/// `n` times.
pub fn repeated<C, T>(elem: T, n: usize) -> C
where T: Clone,
      C: FromIterator<T> {
  iter::repeat(elem).take(n).collect()
}

/// Reduces a double-ended iterator from the right. Similar to
/// `reduce`, this function returns `None` if given an empty iterator.
pub fn reduce_right<I, F>(iter: I, mut f: F) -> Option<I::Item>
where I: DoubleEndedIterator,
      F: FnMut(I::Item, I::Item) -> I::Item {
  iter.rev().reduce(|a, b| f(b, a))
}

/// Like `Iterator::reduce`, but produces an iterator of intermediate
/// values. Reduces left-to-right.
pub fn accum_left<I, F>(iter: I, mut f: F) -> impl Iterator<Item = I::Item>
where I: Iterator,
      F: FnMut(I::Item, I::Item) -> I::Item,
      I::Item: Clone {
  iter.scan(None, move |state, x| {
    let new_state = match state.take() {
      None => x,
      Some(s) => f(s, x),
    };
    *state = Some(new_state.clone());
    Some(new_state)
  })
}

/// Like [`reduce_right`] but produces an iterator of intermediate
/// values. Note that the iterator is produced in reverse order.
///
/// Example:
///
/// ```
/// let v = vec![1, 2, 3];
/// let res: Vec<_> = accum_right(v, |a, b| a + b).collect();
/// assert_eq!(res, vec![3, 5, 6]);
/// ```
pub fn accum_right<I, F>(iter: I, mut f: F) -> impl Iterator<Item = I::Item>
where I: DoubleEndedIterator,
      F: FnMut(I::Item, I::Item) -> I::Item,
      I::Item: Clone {
  accum_left(iter.rev(), move |a, b| f(b, a))
}

/// If the iterator consists of exactly one value, returns that value.
/// Otherwise, returns `None`.
pub fn into_singleton<I: IntoIterator>(iter: I) -> Option<I::Item> {
  let mut iter = iter.into_iter();
  let first_elem = iter.next()?;
  if iter.next().is_none() {
    Some(first_elem)
  } else {
    None
  }
}

/// Converts an iterator into an ordered collection of its elements.
pub fn into_ordered<I: IntoIterator>(iter: I) -> Vec<I::Item>
where I::Item: Ord {
  let mut res = iter.into_iter().collect::<Vec<_>>();
  res.sort();
  res
}

/// Zips two arrays of the same length together, using the given
/// function.
pub fn zip_with<const C: usize, T, S, U, F>(left: [T; C], right: [S; C], mut f: F) -> [U; C]
where F: FnMut(T, S) -> U {
  let res = left.into_iter().zip(right)
    .map(|(x, y)| f(x, y))
    .collect::<Vec<_>>()
    .try_into();
  match res {
    Ok(res) => res,
    Err(_) => panic!("Invalid array length"),
  }
}

/// Splits an iterable of `Either` into a collection of `Left` and a
/// collection of `Right`.
pub fn partition_either<I, A, B, C1, C2>(iter: I) -> (C1, C2)
where I: IntoIterator<Item = Either<A, B>>,
      C1: Default + Extend<A>,
      C2: Default + Extend<B> {
  let mut c1 = C1::default();
  let mut c2 = C2::default();
  for elem in iter {
    match elem {
      Either::Left(a) => c1.extend([a]),
      Either::Right(b) => c2.extend([b]),
    }
  }
  (c1, c2)
}

pub fn partition_mapped<I, T, A, B, F, C1, C2>(iter: I, f: F) -> (C1, C2)
where I: IntoIterator<Item = T>,
      F: FnMut(T) -> Either<A, B>,
      C1: Default + Extend<A>,
      C2: Default + Extend<B> {
  partition_either(iter.into_iter().map(f))
}

/// If the collection is non-empty and all elements of the collection
/// are equal (under `PartialEq`), returns the first element of the
/// collection. If not, returns `None`.
pub fn uniq_element<I>(collection: I) -> Option<I::Item>
where I: IntoIterator,
      I::Item: PartialEq {
  let mut iter = collection.into_iter();
  let first_elem = iter.next()?;
  for elem in iter {
    if first_elem != elem {
      return None;
    }
  }
  Some(first_elem)
}

/// Mutably borrows two elements from a mutable slice at the same
/// time. Panics if the two indices are the same, or if either index
/// is out of bounds.
pub fn double_borrow_mut<T>(slice: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
  match i.cmp(&j) {
    Ordering::Equal => {
      panic!("Cannot mutably borrow index {i} twice at the same time");
    }
    Ordering::Greater => {
      let (b, a) = double_borrow_mut(slice, j, i);
      (a, b)
    }
    Ordering::Less => {
      let (left, right) = slice.split_at_mut(j);
      (&mut left[i], &mut right[0])
    }
  }
}

/// Removes a longest suffix from a vector which satisfies the
/// predicate.
pub fn remove_suffix<T, F>(vec: &mut Vec<T>, mut pred: F)
where F: FnMut(&T) -> bool {
  let mut i = vec.len();
  while i > 0 && pred(&vec[i - 1]) {
    i -= 1;
  }
  vec.truncate(i);
}

/// Compares two iterables lexicographically, according to the given
/// ordering function.
///
/// This function is a re-implementation of the same function on
/// `Iterator`, which is only available in Rust nightly.
pub fn cmp_iter_by<I1, I2, F>(iter1: I1, iter2: I2, mut cmp: F) -> Ordering
where I1: IntoIterator,
      I2: IntoIterator,
      F: FnMut(&I1::Item, &I2::Item) -> Ordering {
  let mut iter1 = iter1.into_iter();
  let mut iter2 = iter2.into_iter();
  loop {
    let Some(a) = iter1.next() else { return if iter2.next().is_none() { Ordering::Equal } else { Ordering::Less } };
    let Some(b) = iter2.next() else { return Ordering::Greater };
    let ord = cmp(&a, &b);
    if ord != Ordering::Equal {
      return ord;
    }
  }
}

pub fn transpose<T>(elems: Vec<Vec<T>>) -> Vec<Vec<T>> {
  let mut elems: Vec<_> = elems.into_iter().map(|v| v.into_iter()).collect();
  let mut result = Vec::with_capacity(elems.len());
  loop {
    let next_elems: Vec<_> = elems.iter_mut().filter_map(|v| v.next()).collect();
    if next_elems.is_empty() {
      break;
    } else {
      result.push(next_elems);
    }
  }
  result
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Implements `PartialEq` and `Eq` to always return true.
  struct AlwaysEq(i64);

  impl PartialEq for AlwaysEq {
    fn eq(&self, _: &AlwaysEq) -> bool {
      true
    }
  }

  impl Eq for AlwaysEq {}

  #[test]
  fn unwrap_infallible_unwraps() {
    let res = Ok(1);
    assert_eq!(unwrap_infallible(res), 1);
  }

  #[test]
  fn test_regex_opt() {
    assert!(regex_opt(["foo", "bar"]).is_match("foo"));
    assert!(regex_opt(["foo", "bar"]).is_match("bar"));
    assert!(!regex_opt(["foo", "bar"]).is_match("baz"));
  }

  #[test]
  fn test_regex_opt_contrived() {
    assert!(regex_opt(["(", ")", "**"]).is_match("("));
    assert!(regex_opt(["(", ")", "**"]).is_match(")"));
    assert!(regex_opt(["(", ")", "**"]).is_match("**"));
    assert!(!regex_opt(["(", ")", "**"]).is_match("e"));
    assert!(!regex_opt(["(", ")", "**"]).is_match(""));
  }

  #[test]
  fn test_regex_opt_output() {
    assert_eq!(regex_opt(["foo", "bar"]).to_string(), "(?:foo|bar)");
    assert_eq!(regex_opt(["bar", "foo"]).to_string(), "(?:bar|foo)");
    assert_eq!(regex_opt(["**", "(x"]).to_string(), r"(?:\*\*|\(x)");
  }

  #[test]
  fn test_regex_opt_output_on_different_string_lengths() {
    assert_eq!(regex_opt(["a", "aaa", "aa", "aaaa", "aaaaa"]).to_string(), "(?:aaaaa|aaaa|aaa|aa|a)");
  }

  #[test]
  fn test_regex_opt_with_output() {
    assert_eq!(regex_opt_with(["**", "(x"], |s| format!("^{s}")).to_string(), r"^(?:\*\*|\(x)");
  }

  #[test]
  fn test_count_prefix() {
    let list = vec![0, 1, 2, 3, 4, 5, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5, 6];
    assert_eq!(count_prefix(list.into_iter(), |x| x < 4), 4);
  }

  #[test]
  fn test_count_suffix() {
    let list = vec![0, 1, 2, 3, 4, 5, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5, 6];
    assert_eq!(count_suffix(list.into_iter(), |x| x > 2), 4);
  }

  #[test]
  fn test_count_prefix_suffix_whole_list() {
    let list = vec![0, 1, 2, 3, 10, 20, 30];
    assert_eq!(count_prefix(list.iter(), |_| true), 7);
    assert_eq!(count_suffix(list.iter(), |_| true), 7);
  }

  #[test]
  fn test_count_prefix_suffix_none_of_list() {
    let list = vec![0, 1, 2, 3, 10, 20, 30];
    assert_eq!(count_prefix(list.iter(), |_| false), 0);
    assert_eq!(count_suffix(list.iter(), |_| false), 0);
  }

  #[test]
  fn test_reduce_right_on_associative_operation() {
    let list = vec![0, 1, 2, 3, 4, 5];
    assert_eq!(reduce_right(list.into_iter(), |a, b| a + b), Some(15));
  }

  #[test]
  fn test_reduce_right_on_empty_collection() {
    let list: Vec<i64> = vec![];
    assert_eq!(reduce_right(list.into_iter(), |_, _| panic!("Should not be called!")), None);
  }

  #[test]
  fn test_reduce_right_on_non_associative_operation() {
    let list = vec![0, 1, 2, 3, 4, 5];
    assert_eq!(reduce_right(list.into_iter(), |a, b| a - b), Some(-3));
  }

  #[test]
  fn test_reduce_right_on_non_associative_operation_with_singleton_list() {
    let list = vec![66];
    assert_eq!(reduce_right(list.into_iter(), |_, _| panic!("Should not be called!")), Some(66));
  }

  #[test]
  fn test_accum_left() {
    let list = vec![1, 2, 3, 4];
    let result: Vec<_> = accum_left(list.into_iter(), |a, b| a - b).collect();
    assert_eq!(result, vec![1, -1, -4, -8]);
  }

  #[test]
  fn test_accum_left_on_empty() {
    let list = Vec::<i64>::new();
    let result: Vec<_> = accum_left(list.into_iter(), |_, _| panic!("Should not be called")).collect();
    assert_eq!(result, Vec::<i64>::new());
  }

  #[test]
  fn test_accum_right() {
    let list = vec![1, 2, 3, 4];
    let result: Vec<_> = accum_right(list.into_iter(), |a, b| a - b).collect();
    assert_eq!(result, vec![4, -1, 3, -2]);
  }

  #[test]
  fn test_accum_right_on_empty() {
    let list = Vec::<i64>::new();
    let result: Vec<_> = accum_right(list.into_iter(), |_, _| panic!("Should not be called")).collect();
    assert_eq!(result, Vec::<i64>::new());
  }

  #[test]
  fn test_into_singleton_on_empty() {
    let list: [i64; 0] = [];
    assert_eq!(into_singleton(list), None);
  }

  #[test]
  fn test_into_singleton() {
    assert_eq!(into_singleton([10]), Some(10));
    assert_eq!(into_singleton([10, 20]), None);
    assert_eq!(into_singleton([10, 20, 30]), None);
    assert_eq!(into_singleton([10, 10, 10, 10, 10]), None);
    assert_eq!(into_singleton(iter::repeat(0)), None);
  }

  #[test]
  fn test_uniq_element() {
    assert_eq!(uniq_element([1, 1, 1, 1]), Some(1));
    assert_eq!(uniq_element(Vec::<i32>::new()), None);
    assert_eq!(uniq_element([1, 1, 1, 2]), None);
  }

  #[test]
  fn test_uniq_element_returns_first_elem() {
    // uniq_element() should always return the first element when it
    // successfully matches. We can test this by implementing a
    // contrived (but lawful) PartialEq on a custom type.
    let elem = uniq_element([AlwaysEq(0), AlwaysEq(10), AlwaysEq(20)]).unwrap();
    assert_eq!(elem.0, 0);
  }

  #[test]
  fn test_double_borrow_mut() {
    let mut arr = [30, 40, 50];
    let (a, b) = double_borrow_mut(&mut arr, 1, 2);
    assert_eq!(a, &mut 40);
    assert_eq!(b, &mut 50);
    let (a, b) = double_borrow_mut(&mut arr, 2, 1);
    assert_eq!(a, &mut 50);
    assert_eq!(b, &mut 40);
  }

  #[test]
  #[should_panic]
  fn test_double_borrow_mut_panic() {
    let mut arr = [30, 40, 50];
    double_borrow_mut(&mut arr, 0, 0);
  }

  #[test]
  fn test_remove_suffix_vec() {
    let mut vec = vec![10, 20, 30, 40, 50];
    remove_suffix(&mut vec, |x| *x > 30);
    assert_eq!(vec, &[10, 20, 30]);
  }

  #[test]
  fn test_remove_suffix_vec_no_op() {
    let mut vec = vec![10, 20, 30, 40, 50];
    remove_suffix(&mut vec, |x| *x < 30);
    assert_eq!(vec, &[10, 20, 30, 40, 50]);
  }

  #[test]
  fn test_remove_suffix_vec_all_elements() {
    let mut vec = vec![10, 20, 30, 40, 50];
    remove_suffix(&mut vec, |x| *x >= 0);
    assert_eq!(vec, Vec::<i64>::new());
  }

  #[test]
  fn test_remove_suffix_vec_on_empty_vec() {
    let mut vec = Vec::<i64>::new();
    remove_suffix(&mut vec, |_| panic!("Should not be called"));
    assert_eq!(vec, Vec::<i64>::new());
  }

  #[test]
  fn test_cmp_iter_by_equal() {
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    assert_eq!(cmp_iter_by(&a, &b, |_, _| Ordering::Equal), Ordering::Equal);
  }

  #[test]
  fn test_cmp_iter_by_equal_with_one_longer_vec() {
    let a = vec![1, 2, 3, 10];
    let b = vec![4, 5, 6];
    assert_eq!(cmp_iter_by(&a, &b, |_, _| Ordering::Equal), Ordering::Greater);

    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6, 10];
    assert_eq!(cmp_iter_by(&a, &b, |_, _| Ordering::Equal), Ordering::Less);
  }

  #[test]
  fn test_cmp_iter_by_empty() {
    let a: Vec<i64> = vec![];
    let b: Vec<i64> = vec![10, 20];
    assert_eq!(cmp_iter_by(&a, &b, |_, _| unreachable!()), Ordering::Less);
    assert_eq!(cmp_iter_by(&b, &a, |_, _| unreachable!()), Ordering::Greater);
  }

  #[test]
  fn test_cmp_iter_by() {
    let a: Vec<i64> = vec![1, 2, 3, 4];
    let b: Vec<i64> = vec![1, -2, 3, -5];
    assert_eq!(cmp_iter_by(&a, &b, |x, y| x.abs().cmp(&y.abs())), Ordering::Less);
    assert_eq!(cmp_iter_by(&b, &a, |x, y| x.abs().cmp(&y.abs())), Ordering::Greater);
  }

  #[test]
  fn test_transpose_on_square_matrix() {
    let vec = vec![
      vec![0, 1, 2, 3],
      vec![4, 5, 6, 7],
      vec![8, 9, 10, 11],
      vec![12, 13, 14, 15],
    ];
    assert_eq!(transpose(vec), vec![
      vec![0, 4, 8, 12],
      vec![1, 5, 9, 13],
      vec![2, 6, 10, 14],
      vec![3, 7, 11, 15],
    ]);
  }

  #[test]
  fn test_transpose_on_rectangular_matrix() {
    let vec = vec![
      vec![0, 1, 2, 3],
      vec![4, 5, 6, 7],
    ];
    assert_eq!(transpose(vec), vec![
      vec![0, 4],
      vec![1, 5],
      vec![2, 6],
      vec![3, 7],
    ]);
  }

  #[test]
  fn test_transpose_on_jagged_array() {
    let vec = vec![
      vec![0, 1, 2, 3],
      vec![4, 5, 6, 7, 8],
      vec![9, 10],
      vec![11, 12, 13],
    ];
    assert_eq!(transpose(vec), vec![
      vec![0, 4, 9, 11],
      vec![1, 5, 10, 12],
      vec![2, 6, 13],
      vec![3, 7],
      vec![8],
    ]);
  }
}
