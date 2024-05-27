
//! Various utility functions.

pub mod angles;
pub mod prism;
pub mod stricteq;

use regex::{Regex, escape};

use std::convert::Infallible;

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
  let regex_str = options.into_iter().map(|s| escape(s)).collect::<Vec<_>>().join("|");
  let regex_str = helper(format!("(?:{regex_str})"));
  Regex::new(&regex_str).expect(&format!("Invalid regular expression: {}", regex_str))
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
    assert_eq!(regex_opt(["**", "(x"]).to_string(), r"(?:\*\*|\(x)");
  }

  #[test]
  fn test_regex_opt_with_output() {
    assert_eq!(regex_opt_with(["**", "(x"], |s| format!("^{s}")).to_string(), r"^(?:\*\*|\(x)");
  }
}
