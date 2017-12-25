/// Incrementable is a trait to be implemented when supporting
/// increment operations.
///
/// Incrementable is copied from the excellent Stackoverflow answer
/// https://stackoverflow.com/a/41671697/267196
///
pub trait Incrementable: Copy + ::std::ops::AddAssign<Self> {
    /// Generic incrementaton step, eg 1 for i32
    fn one() -> Self;

    /// Increment by one after use of value
    fn post_inc(&mut self) -> Self {
        self.post_inc_by(Self::one())
    }

    /// Increment by n after use of value
    fn post_inc_by(&mut self, n: Self) -> Self {
        let tmp = *self;
        *self += n;
        tmp
    }
}

/// A macro helps to implement it for numeric types
macro_rules! impl_Incrementable{
    ($($m:ty),*) => {$( impl Incrementable for $m  { fn one() -> Self { 1 as $m } })*}
}
impl_Incrementable!{u8, u16, u32, u64, i8, i16, i32, i64, f32, f64}
