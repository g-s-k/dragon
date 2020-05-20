use std::{
    cell::RefCell,
    fmt,
    io::{self, Error, ErrorKind, Read},
    iter::Peekable,
    mem,
    rc::Rc,
};

fn main() -> EmptyIoResult {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;

    let mut symbols = SymTable::default();
    symbols.insert("div".to_string());
    symbols.insert("mod".to_string());

    let lexer = Lexer::new(s.chars(), symbols.clone());

    Parser {
        iter: lexer.peekable(),
        symbols,
    }
    .list()
}

type EmptyIoResult = io::Result<()>;

#[derive(Clone, Default)]
struct SymTable(Rc<RefCell<Vec<String>>>);

impl SymTable {
    fn get(&self, index: usize) -> Option<String> {
        self.0.borrow().get(index).cloned()
    }

    fn lookup(&self, sym: &str) -> Option<usize> {
        self.0.borrow().iter().position(|s| s == sym)
    }

    fn insert(&mut self, sym: String) -> usize {
        if let Some(idx) = self.lookup(&sym) {
            idx
        } else {
            self.0.borrow_mut().push(sym);
            self.0.borrow().len() - 1
        }
    }
}

pub(crate) struct Lexer<I: Iterator<Item = char>> {
    iter: Peekable<I>,
    line: usize,
    symbols: SymTable,
}

impl<I: Iterator<Item = char>> Lexer<I> {
    pub(crate) fn new(iter: I, symbols: SymTable) -> Self {
        Self {
            iter: iter.peekable(),
            line: 1,
            symbols,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
    type Item = FallibleToken;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.iter.next() {
            match c {
                ' ' | '\t' => (),
                '\n' => self.line += 1,
                c @ '0'..='9' => {
                    let mut num = (c as u8 - b'0') as usize;

                    'num: loop {
                        match self.iter.peek() {
                            None => return None,
                            Some(c @ '0'..='9') => {
                                num *= 10;
                                num += (*c as u8 - b'0') as usize;
                                self.iter.next();
                            }
                            _ => break 'num,
                        }
                    }

                    return Some((Ok(Token::Num(num)), self.line));
                }
                '+' => return Some((Ok(Token::Plus), self.line)),
                '-' => return Some((Ok(Token::Minus), self.line)),
                '*' => return Some((Ok(Token::Times), self.line)),
                '/' => return Some((Ok(Token::Div), self.line)),
                '(' => return Some((Ok(Token::LParen), self.line)),
                ')' => return Some((Ok(Token::RParen), self.line)),
                ';' => return Some((Ok(Token::Semi), self.line)),
                c if c.is_alphabetic() => {
                    let mut ident = String::new();
                    ident.push(c);

                    'ident: loop {
                        match self.iter.peek() {
                            None => return None,
                            Some(&c) if c.is_alphanumeric() => {
                                ident.push(c);
                                self.iter.next();
                            }
                            _ => break 'ident,
                        }
                    }

                    let idx = self.symbols.insert(ident);
                    return Some((Ok(Token::Sym(idx)), self.line));
                }
                _ => return Some((Err(c), self.line)),
            }
        }

        None
    }
}

type FallibleToken = (Result<Token, char>, usize);

#[derive(Clone, Copy, Debug)]
enum Token {
    Plus,
    Minus,
    Times,
    Div,
    LParen,
    RParen,
    Semi,

    Num(usize),
    Sym(usize),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Times => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::Semi => write!(f, ";"),
            Self::Num(n) => write!(f, "{}", n),
            Self::Sym(i) => write!(f, "<symbol {}>", i),
        }
    }
}

struct Parser<I: Iterator<Item = FallibleToken>> {
    iter: Peekable<I>,
    symbols: SymTable,
}

impl<I> Parser<I>
where
    I: Iterator<Item = FallibleToken>,
{
    fn list(&mut self) -> EmptyIoResult {
        while self.peek().is_some() {
            self.expr()?;
            self._match(Token::Semi)?;
        }

        Ok(())
    }

    fn expr(&mut self) -> EmptyIoResult {
        self.term()?;

        while let Some(t) = self.peek() {
            match t? {
                Token::Plus => {
                    self._match(Token::Plus)?;
                    self.term()?;
                    println!("+")
                }
                Token::Minus => {
                    self._match(Token::Minus)?;
                    self.term()?;
                    println!("-")
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn term(&mut self) -> EmptyIoResult {
        self.factor()?;

        while let Some(t) = self.peek() {
            match t? {
                Token::Times => {
                    self._match(Token::Times)?;
                    self.factor()?;
                    println!("*")
                }
                Token::Div => {
                    self._match(Token::Div)?;
                    self.factor()?;
                    println!("/")
                }
                Token::Sym(s) => match self.resolve_sym(s)?.as_ref() {
                    "div" => {
                        self._match(Token::Sym(0))?;
                        self.factor()?;
                        println!("DIV")
                    }
                    "mod" => {
                        self._match(Token::Sym(0))?;
                        self.factor()?;
                        println!("MOD")
                    }
                    _ => break,
                },
                _ => break,
            }
        }

        Ok(())
    }

    fn factor(&mut self) -> EmptyIoResult {
        match self.peek_non_null()? {
            Token::LParen => {
                self._match(Token::LParen)?;
                self.expr()?;
                self._match(Token::RParen)
            }
            Token::Num(n) => {
                println!("{}", n);
                self._match(Token::Num(0))
            }
            Token::Sym(s) => {
                let sym = self.resolve_sym(s)?;
                println!("{}", sym);
                self._match(Token::Sym(0))
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "expected a number or parenthesized expression",
            )),
        }
    }

    fn peek_non_null(&mut self) -> io::Result<Token> {
        self.peek()
            .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, ""))?
    }

    fn peek(&mut self) -> Option<io::Result<Token>> {
        match self.iter.peek() {
            Some((Ok(typ), _)) => Some(Ok(*typ)),
            Some((Err(c), line)) => Some(Err(Error::new(
                ErrorKind::InvalidData,
                format!("unexpected character `{}` on line {} of input", c, line),
            ))),

            None => None,
        }
    }

    fn _match(&mut self, c: Token) -> EmptyIoResult {
        if mem::discriminant(&self.peek_non_null()?) == mem::discriminant(&c) {
            self.iter.next();
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("expected the token `{}`.", c),
            ))
        }
    }

    fn resolve_sym(&self, symbol_index: usize) -> io::Result<String> {
        self.symbols.get(symbol_index).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("no symbol in table at index {}", symbol_index),
            )
        })
    }
}
