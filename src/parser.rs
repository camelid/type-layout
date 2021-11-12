mod scope;

use std::iter::Peekable;

use crate::hir::{Expr, Pat, Ty, Var};
use crate::name::Name;
use crate::util::Map;

use self::scope::{Scope, ScopeStack};

#[derive(Debug)]
pub struct Parser {
    tokens: Peekable<Tokenizer>,
    aliases: Map<Name, Ty>,
    ty_scopes: ScopeStack,
}

impl Parser {
    pub fn parse(src: String) -> Expr {
        let mut parser = Self::new(src);
        let expr = parser.parse_expr();
        parser.expect_end();
        expr
    }

    pub fn parse_ty_toplevel(src: String) -> Ty {
        let mut parser = Self::new(src);
        let ty = parser.parse_ty();
        parser.expect_end();
        ty
    }

    fn new(src: String) -> Self {
        Parser {
            tokens: Tokenizer::new(src).peekable(),
            aliases: Map::new(),
            ty_scopes: ScopeStack::empty(),
        }
    }

    fn parse_expr(&mut self) -> Expr {
        match self.bump() {
            Token::LParen => self.parse_expr_grouping(),
            Token::LBrace => self.parse_expr_record(),
            Token::LAngle => self.parse_expr_variant(),
            Token::KwFold => self.parse_expr_fold(),
            Token::KwUnfold => self.parse_expr_unfold(),
            Token::KwBoxOp => self.parse_expr_box(),
            Token::KwLet => self.parse_expr_let(),
            Token::KwAlias => {
                self.parse_alias();
                self.parse_expr()
            }
            Token::KwMatch => self.parse_expr_match(),
            Token::Number(text) => {
                let u = text.parse().unwrap_or_else(|_| {
                    error(format!("invalid number: {}", text.escape_default()))
                });
                Expr::U64(u)
            }
            Token::Ident(name) => Expr::Var(self.parse_var_after_name(Name::from(name))),
            tok => error(format!("expected expression, found {}", tok)),
        }
    }

    fn parse_alias(&mut self) {
        let name = self.parse_name();
        self.eat(Token::Eq);
        let defn = self.parse_ty();
        self.aliases.insert(name, defn);
        self.eat(Token::KwIn);
    }

    fn parse_expr_grouping(&mut self) -> Expr {
        let e = self.parse_expr();
        self.eat(Token::RParen);
        e
    }

    fn parse_expr_record(&mut self) -> Expr {
        let mut fields = map! {};
        self.parse_delimited(Token::Comma, Token::RBrace, |this| {
            let name = this.parse_name();
            this.eat(Token::Eq);
            let value = this.parse_expr();
            fields.insert(name, value);
        });
        Expr::Record(fields)
    }

    fn parse_expr_variant(&mut self) -> Expr {
        let variant = self.parse_name();
        self.eat(Token::Eq);
        let field = self.parse_expr();
        self.eat(Token::RAngle);
        self.eat(Token::KwAs);
        let ty = self.parse_ty();
        Expr::Variant { ty, variant, field: Box::new(field) }
    }

    fn parse_expr_fold(&mut self) -> Expr {
        self.eat(Token::LBracket);
        let ty = self.parse_ty();
        self.eat(Token::RBracket);
        let value = self.parse_expr();
        Expr::Fold { ty, value: Box::new(value) }
    }

    fn parse_expr_unfold(&mut self) -> Expr {
        self.eat(Token::LBracket);
        let ty = self.parse_ty();
        self.eat(Token::RBracket);
        let value = self.parse_expr();
        Expr::Unfold { ty, value: Box::new(value) }
    }

    fn parse_expr_box(&mut self) -> Expr {
        self.eat(Token::LParen);
        let boxed = self.parse_expr();
        self.eat(Token::RParen);
        Expr::Box(Box::new(boxed))
    }

    fn parse_expr_let(&mut self) -> Expr {
        let binder = self.parse_var();
        self.eat(Token::Eq);
        let value = self.parse_expr();
        self.eat(Token::KwIn);
        let body = self.parse_expr();
        Expr::Let { binder, value: Box::new(value), body: Box::new(body) }
    }

    fn parse_expr_match(&mut self) -> Expr {
        let subj = self.parse_expr();

        let mut cases = vec![];
        self.eat(Token::LBrace);
        self.parse_delimited(Token::Comma, Token::RBrace, |this| {
            let pat = this.parse_pat();
            this.eat(Token::WideArrow);
            let body = this.parse_expr();
            cases.push((pat, body));
        });

        Expr::Match { subj: Box::new(subj), cases }
    }

    fn parse_pat(&mut self) -> Pat {
        self.eat(Token::LAngle);
        self.parse_variant_pat()
    }

    fn parse_variant_pat(&mut self) -> Pat {
        let variant = self.parse_name();
        self.eat(Token::Eq);
        let field = self.parse_var();
        self.eat(Token::RAngle);
        self.eat(Token::KwAs);
        let ty = self.parse_ty();
        Pat::Variant { ty, variant, field }
    }

    fn parse_var(&mut self) -> Var {
        let name = self.parse_name();
        self.parse_var_after_name(name)
    }

    fn parse_var_after_name(&mut self, name: Name) -> Var {
        self.eat_msg(Token::Colon, "type annotation after variable name");
        let ty = self.parse_ty();
        Var { name, ty }
    }

    fn parse_ty(&mut self) -> Ty {
        match self.bump() {
            Token::KwBoxTy => self.parse_ty_box(),
            Token::LBrace => self.parse_ty_record(),
            Token::LAngle => self.parse_ty_variant(),
            Token::KwMu => self.parse_ty_recur(),
            Token::Ident(s) => {
                let name = Name::from(s);
                self.aliases
                    .get(&name)
                    .cloned()
                    .or_else(|| self.ty_scopes.lookup(&name).map(Ty::Named))
                    .or_else(|| if name.as_user() == Some("U64") { Some(Ty::U64) } else { None })
                    .unwrap_or_else(|| error(format!("name not found: {}", name)))
            }
            tok => error(format!("expected type, found {}", tok)),
        }
    }

    fn parse_ty_box(&mut self) -> Ty {
        self.eat(Token::LBracket);
        let boxed = self.parse_ty();
        self.eat(Token::RBracket);
        Ty::Box(Box::new(boxed))
    }

    fn parse_ty_record(&mut self) -> Ty {
        let mut fields = map! {};

        self.parse_delimited(Token::Comma, Token::RBrace, |this| {
            let name = this.parse_name();
            this.eat(Token::Colon);
            let ty = this.parse_ty();
            fields.insert(name, ty);
        });

        Ty::Record(fields)
    }

    fn parse_ty_variant(&mut self) -> Ty {
        let mut variants = map! {};

        self.parse_delimited(Token::VertPipe, Token::RAngle, |this| {
            let name = this.parse_name();
            this.eat(Token::KwOf);
            let ty = this.parse_ty();
            variants.insert(name, ty);
        });

        Ty::Variant(variants)
    }

    fn parse_ty_recur(&mut self) -> Ty {
        let binding = self.parse_name();
        self.push_type_scope(binding);

        self.eat(Token::Dot);
        let body = self.parse_ty();

        self.pop_type_scope();
        Ty::Recursive(Box::new(body))
    }

    fn push_type_scope(&mut self, binding: Name) {
        self.ty_scopes.push(Scope::new(binding))
    }

    fn pop_type_scope(&mut self) {
        self.ty_scopes.pop()
    }

    fn parse_name(&mut self) -> Name {
        match self.bump() {
            Token::Ident(s) => Name::from(s),
            tok => error(format!("expected name, found {}", tok)),
        }
    }

    fn parse_delimited(&mut self, delim: Token, end: Token, parse_elem: impl FnMut(&mut Self)) {
        self.parse_delimited_until(delim, end.clone(), parse_elem);
        self.eat(end);
    }

    fn parse_delimited_until(
        &mut self,
        delim: Token,
        until: Token,
        mut parse_elem: impl FnMut(&mut Self),
    ) {
        while !self.check(until.clone()) {
            parse_elem(self);

            if !self.check(until.clone()) {
                self.eat(delim.clone());
            }
        }
    }

    fn expect_end(&mut self) {
        match self.peek() {
            Some(tok) => error(format!("expected end, found {}", tok)),
            None => {}
        }
    }

    fn eat(&mut self, expect: Token) {
        let msg = expect.to_string();
        self.eat_msg(expect, &msg)
    }

    fn eat_msg(&mut self, expect: Token, msg: &str) {
        match self.tokens.next() {
            Some(tok) if tok == expect => {}
            Some(tok) => error(format!("expected {}, found {}", msg, tok)),
            None => error(format!("expected {}", msg)),
        }
    }

    fn bump(&mut self) -> Token {
        match self.tokens.next() {
            Some(tok) => tok,
            None => error("unexpected end".into()),
        }
    }

    fn check(&mut self, expect: Token) -> bool {
        match self.peek() {
            Some(tok) => *tok == expect,
            _ => false,
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }
}

#[derive(Debug)]
struct Tokenizer {
    src: Vec<char>,
    start: usize,
    current: usize,
}

impl Tokenizer {
    fn new(src: String) -> Self {
        Self { src: src.chars().collect(), start: 0, current: 0 }
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();

        self.start = self.current;

        if self.peek().is_none() {
            return None;
        }

        match self.bump().unwrap() {
            '=' if self.check('>') => {
                self.bump().unwrap();
                Some(Token::WideArrow)
            }
            '=' => Some(Token::Eq),
            ':' => Some(Token::Colon),
            ',' => Some(Token::Comma),
            ';' => Some(Token::Semi),
            '.' => Some(Token::Dot),
            '|' => Some(Token::VertPipe),

            '(' => Some(Token::LParen),
            ')' => Some(Token::RParen),
            '[' => Some(Token::LBracket),
            ']' => Some(Token::RBracket),
            '{' => Some(Token::LBrace),
            '}' => Some(Token::RBrace),
            '<' => Some(Token::LAngle),
            '>' => Some(Token::RAngle),

            '0'..='9' => {
                while self.check_fn(|c| ('0'..='9').contains(&c)) {
                    self.bump();
                }
                let lexeme = self.mk_lexeme();
                Some(Token::Number(lexeme))
            }

            'µ' => Some(Token::KwMu),
            chr if chr.is_ascii_alphabetic() || chr == '_' => {
                while self.check_fn(|c| c.is_ascii_alphanumeric() || c == '_') {
                    self.bump();
                }
                let lexeme = self.mk_lexeme();
                match lexeme.as_str() {
                    "let" => Some(Token::KwLet),
                    "in" => Some(Token::KwIn),
                    "match" => Some(Token::KwMatch),
                    "of" => Some(Token::KwOf),
                    "as" => Some(Token::KwAs),
                    "alias" => Some(Token::KwAlias),
                    "fold" => Some(Token::KwFold),
                    "unfold" => Some(Token::KwUnfold),
                    "Box" => Some(Token::KwBoxTy),
                    "box" => Some(Token::KwBoxOp),
                    _ => Some(Token::Ident(lexeme)),
                }
            }

            c => error(format!("unexpected char: {}", c.escape_default())),
        }
    }
}

impl Tokenizer {
    fn skip_ws(&mut self) {
        while let Some(chr) = self.peek() {
            match chr {
                ' ' | '\t' | '\n' | '\r' => {
                    self.bump().unwrap();
                }
                '-' if self.peek_next() == Some('-') => {
                    self.bump().unwrap();
                    self.bump().unwrap();
                    while self.check_fn(|c| c != '\n') {
                        self.bump().unwrap();
                    }
                }
                _ => {
                    break;
                }
            }
        }
    }

    fn bump(&mut self) -> Option<char> {
        let chr = self.peek()?;
        self.current += 1;
        Some(chr)
    }

    fn check(&self, is: char) -> bool {
        self.check_fn(|c| c == is)
    }

    fn check_fn(&self, f: impl Fn(char) -> bool) -> bool {
        self.peek().map(f).unwrap_or(false)
    }

    fn peek_next(&self) -> Option<char> {
        self.src.get(self.current + 1).copied()
    }

    fn peek(&self) -> Option<char> {
        self.src.get(self.current).copied()
    }

    fn mk_lexeme(&self) -> String {
        self.src[self.start..self.current].iter().collect()
    }
}

// TODO: handle properly instead of panicking
#[track_caller]
fn error(msg: String) -> ! {
    eprintln!("syntax error: {}", msg);
    panic!("syntax error encountered")
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    // Single-character symbols.
    Eq,
    Colon,
    Comma,
    Semi,
    Dot,
    VertPipe,

    // Matching single-character symbols.
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    LAngle,
    RAngle,

    // Multi-character symbols.
    WideArrow,

    // Keywords.
    KwLet,
    KwIn,
    KwMatch,
    KwOf,
    KwAs,
    KwAlias,
    // TODO: use "rec" or "recur" instead?
    KwMu,
    KwFold,
    KwUnfold,
    KwBoxTy,
    KwBoxOp,

    Number(String),
    Ident(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "`{}`",
            match self {
                Token::Eq => "=",
                Token::Colon => ":",
                Token::Comma => ",",
                Token::Semi => ";",
                Token::Dot => ".",
                Token::VertPipe => "|",
                Token::LParen => "(",
                Token::RParen => ")",
                Token::LBracket => "[",
                Token::RBracket => "]",
                Token::LBrace => "{",
                Token::RBrace => "}",
                Token::LAngle => "<",
                Token::RAngle => ">",
                Token::WideArrow => "=>",
                Token::KwLet => "let",
                Token::KwIn => "in",
                Token::KwMatch => "match",
                Token::KwOf => "of",
                Token::KwAs => "as",
                Token::KwAlias => "alias",
                Token::KwMu => "µ",
                Token::KwFold => "fold",
                Token::KwUnfold => "unfold",
                Token::KwBoxTy => "Box",
                Token::KwBoxOp => "box",
                Token::Number(s) => s,
                Token::Ident(s) => s,
            }
        )
    }
}
