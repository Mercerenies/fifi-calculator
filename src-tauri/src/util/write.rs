
use std::fmt::{self, Write, Arguments};
use std::convert::Infallible;

/// Newtype wrapper which implements [`SafeWrite`] for any type which
/// implements [`std::fmt::Write`].
#[derive(Debug, Clone)]
pub struct WriteAsSafeWrite<W>(pub W);

/// This trait is equivalent to [`std::fmt::Write`] except that the
/// produced error type is an associated type. This guarantees, at
/// compile-time, that calling `write_fmt` on a type that never errs,
/// such as `String`, will produce a type with no error component.
pub trait SafeWrite {
  type Error;

  fn write_str(&mut self, s: &str) -> Result<(), Self::Error>;
  fn write_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error>;

  fn write_char(&mut self, c: char) -> Result<(), Self::Error> {
    self.write_str(c.encode_utf8(&mut [0; 4]))
  }
}

impl SafeWrite for String {
  type Error = Infallible;

  fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
    self.push_str(s);
    Ok(())
  }

  fn write_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error> {
    Write::write_fmt(self, args).map_err(self_write_never_panics)
  }
}

impl<W: SafeWrite> SafeWrite for &mut W {
  type Error = W::Error;

  fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
    W::write_str(*self, s)
  }

  fn write_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error> {
    W::write_fmt(*self, args)
  }
}

impl<W: Write> SafeWrite for WriteAsSafeWrite<W> {
  type Error = fmt::Error;

  fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
    self.0.write_str(s)
  }

  fn write_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error> {
    self.0.write_fmt(args)
  }
}

/// An assertion that the error in question will never occur. If this
/// function actually ends up getting called, it always panics.
fn self_write_never_panics(_: fmt::Error) -> Infallible {
  panic!("Expected std::fmt::Write to never fail, but it failed");
}
