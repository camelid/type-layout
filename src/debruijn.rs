#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Debruijn {
    index: u64,
}

impl Debruijn {
    pub const ZERO: Self = Self::new(0);

    pub const fn new(index: u64) -> Self {
        Self { index }
    }

    pub const fn index(self) -> u64 {
        self.index
    }

    pub fn shift_by(mut self, by: u64) -> Self {
        self.index = self.index.checked_add(by).unwrap();
        self
    }
}

impl std::fmt::Display for Debruijn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { index } = self;
        write!(f, "#{}", index)
    }
}
