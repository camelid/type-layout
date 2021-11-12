use super::Ty;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size {
    bytes: u64,
}

impl Size {
    pub const ZERO: Self = Self::from_bytes(0);

    pub const BITS_64: Self = Self::from_bytes(8);

    pub const fn from_bytes(bytes: u64) -> Self {
        Self { bytes }
    }

    pub fn bytes(self) -> u64 {
        self.bytes
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { bytes } = self;
        write!(f, "{} byte{}", bytes, if *bytes == 1 { "" } else { "s" })
    }
}

impl Ty {
    /// The "packed" size of a type---i.e., the size of the actual data, ignoring padding.
    ///
    /// This should correspond at least roughly to the Swift notion of size (not stride).
    pub fn packed_size(&self) -> Size {
        match self {
            Ty::U64 => Size::BITS_64,
            Ty::Ptr(_) => Size::BITS_64,
            Ty::Record(fields) => {
                let sizes = fields.values().map(|t| t.packed_size().bytes());
                Size::from_bytes(sizes.sum())
            }
            Ty::UntaggedUnion(variants) => {
                let sizes = variants.values().map(|t| t.packed_size().bytes());
                sizes.max().map(Size::from_bytes).unwrap_or(Size::ZERO)
            }
            Ty::Recursive(body) => body.packed_size(),
            Ty::RecurId(_) => todo!(),
        }
    }
}
