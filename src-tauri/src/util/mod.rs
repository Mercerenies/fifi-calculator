
//! Various utility functions.

pub mod angles;
pub mod matrix;
pub mod point;
pub mod prism;
pub mod stricteq;

use regex::{Regex, escape};
use either::Either;

use std::convert::Infallible;
use std::cmp::Reverse;
use std::iter::{self, Extend};

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

#[cfg(test)]
mod tests {
  use super::*;

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
}
