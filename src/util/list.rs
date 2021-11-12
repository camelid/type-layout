use std::mem;

pub use List::{Cons, Nil};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Nil
    }
}

impl<T> List<T> {
    pub fn singleton(x: T) -> List<T> {
        Cons(x, Box::new(Nil))
    }

    pub fn concat(mut self, other: List<T>) -> List<T> {
        match &mut self {
            Nil => other,
            Cons(_, tl_ref) => {
                // This avoids allocating a new box.
                let tl = mem::take(&mut **tl_ref);
                **tl_ref = tl.concat(other);
                self
            }
        }
    }

    pub fn for_each_reverse<F: FnMut(&T)>(self, mut f: F) {
        self._for_each_reverse_impl(&mut f)
    }
    fn _for_each_reverse_impl<F: FnMut(&T)>(self, f: &mut F) {
        match self {
            Nil => {}
            Cons(hd, tl) => {
                tl.for_each_reverse(&mut *f);
                f(&hd);
            }
        }
    }

    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { cur: self }
    }
}

pub struct IntoIter<T> {
    cur: List<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // We need `cur` to be owned, so swap `Nil` in and the actual node out.
        let cur = mem::take(&mut self.cur);
        match cur {
            // If `cur` is `Nil`, then we don't need to do anything because we
            // already put `Nil` in with `mem::take`.
            Nil => None,
            Cons(hd, tl) => {
                // If `cur` is `Cons`, then replace `self.cur` with the tail
                // and return the head.
                self.cur = *tl;
                Some(hd)
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { cur: self }
    }
}

pub struct Iter<'a, T> {
    cur: &'a List<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur {
            Nil => None,
            Cons(hd, tl) => {
                self.cur = tl;
                Some(hd)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_concat() {
        assert_eq!(Nil::<()>.concat(Nil), Nil);
        assert_eq!(Nil.concat(List::singleton(123)), List::singleton(123));
        assert_eq!(List::singleton(123).concat(Nil), List::singleton(123));

        assert_eq!(
            List::singleton(123).concat(List::singleton(456)),
            Cons(123, Box::new(Cons(456, Box::new(Nil))))
        );
        assert_eq!(
            List::singleton(123).concat(List::singleton(456)).concat(List::singleton(789)),
            Cons(123, Box::new(Cons(456, Box::new(Cons(789, Box::new(Nil))))))
        );
        // Different grouping from above.
        assert_eq!(
            List::singleton(123).concat(List::singleton(456).concat(List::singleton(789))),
            Cons(123, Box::new(Cons(456, Box::new(Cons(789, Box::new(Nil))))))
        );
    }
}
