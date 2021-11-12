#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Name {
    User(String),
    Temp(u64),
}

impl From<u32> for Name {
    fn from(n: u32) -> Name {
        Name::User(n.to_string())
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Name {
        Name::User(s.to_owned())
    }
}

impl From<String> for Name {
    fn from(s: String) -> Name {
        Name::User(s)
    }
}

impl Name {
    pub fn as_user(&self) -> Option<&str> {
        match self {
            Name::User(s) => Some(s),
            Name::Temp(_) => None,
        }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // TODO: display as `u#{}`?
            Name::User(text) => write!(f, "{}", text),
            Name::Temp(idx) => write!(f, "t#{}", idx),
        }
    }
}
