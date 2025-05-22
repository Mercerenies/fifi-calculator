
use super::{Prism, Composed, OnTuple2, DisjPrism, Iso};

/// Extension trait which provides prism helpers. Provides a blanket
/// impl for all prisms.
pub trait PrismExt<Up, Down>: Prism<Up, Down> {
  /// Composes two prisms together, with the left prism running first
  /// on narrow and the right running first on widen.
  fn composed<P, Down1>(self, other: P) -> Composed<Self, P, Down>
  where Self: Sized,
        P: Prism<Down, Down1> {
    Composed::new(self, other)
  }

  /// Prism which applies two constituent prisms to the elements of a
  /// 2-tuple.
  fn and<P, Up1, Down1>(self, other: P) -> OnTuple2<Self, P>
  where Self: Sized,
        P: Prism<Up1, Down1> {
    OnTuple2::new(self, other)
  }

  /// Prism which tries `self` and then `other`, taking the first one
  /// which succeeds.
  ///
  /// WARNING: This `Prism` instance is only lawful if the two
  /// constituent prisms are disjoint! See the documentation for
  /// [`DisjPrism`] for more details!
  fn or<P, Down1>(self, other: P) -> DisjPrism<Self, P>
  where Self: Sized,
        P: Prism<Up, Down1> {
    DisjPrism::new(self, other)
  }

  /// Prism which maps its result to a new result type in a
  /// bidirectional way.
  fn rmap<F, G, Down1>(self, narrow: F, widen: G) -> Composed<Self, Iso<Down, Down1, F, G>, Down>
  where F: Fn(Down) -> Down1,
        G: Fn(Down1) -> Down,
        Self: Sized {
    self.composed(Iso::new(narrow, widen))
  }

  /// Prism which maps its preimage to a new input type in a
  /// bidirectional way.
  fn lmap<F, G, Up1>(self, narrow: F, widen: G) -> Composed<Iso<Up1, Up, F, G>, Self, Up>
  where F: Fn(Up1) -> Up,
        G: Fn(Up) -> Up1,
        Self: Sized {
    Iso::new(narrow, widen).composed(self)
  }
}

impl<Up, Down, P> PrismExt<Up, Down> for P
where P: Prism<Up, Down> {}
