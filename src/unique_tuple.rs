// credit: https://users.rust-lang.org/t/make-hash-return-same-value-whather-the-order-of-element-of-a-tuple/69932/4

use std::hash::{DefaultHasher, Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct UniqueTuple<T: Hash>(pub T, pub T);

impl<T: Hash> Hash for UniqueTuple<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(calculate_hash(&self.0) ^ calculate_hash(&self.1));
    }
}

impl<T: Hash> From<(T, T)> for UniqueTuple<T> {
    fn from((a, b): (T, T)) -> Self {
        Self(a, b)
    }
}

impl<T: Hash> Into<(T, T)> for UniqueTuple<T> {
    fn into(self) -> (T, T) {
        (self.0, self.1)
    }
}
