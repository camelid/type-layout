use crate::debruijn::Debruijn;

use crate::name::Name;

#[derive(Debug)]
pub(crate) struct ScopeStack {
    pub(crate) stack: Vec<Scope>,
}

impl ScopeStack {
    pub(crate) fn empty() -> Self {
        Self { stack: vec![] }
    }

    pub(crate) fn push(&mut self, scope: Scope) {
        self.stack.push(scope)
    }

    pub(crate) fn pop(&mut self) {
        self.stack.pop().unwrap();
    }

    pub(crate) fn lookup(&self, name: &Name) -> Option<Debruijn> {
        let scopes = self.stack.iter().rev().zip(0..);
        scopes
            .filter_map(|(scope, shift)| scope.lookup(name).map(|k| (k.index(), shift)))
            .next()
            .map(|(local_index, shift)| Debruijn::new(local_index + shift))
    }
}

#[derive(Debug)]
pub(crate) struct Scope {
    pub(crate) binding: Name,
}

impl Scope {
    pub(crate) fn new(binding: Name) -> Self {
        Self { binding }
    }

    pub(crate) fn lookup(&self, name: &Name) -> Option<Debruijn> {
        if name == &self.binding {
            Some(Debruijn::ZERO)
        } else {
            None
        }
    }
}
