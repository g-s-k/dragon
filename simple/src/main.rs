use std::{
    io::{self, Error, ErrorKind, Read, Write},
    iter::Peekable,
    str::Chars,
};

fn main() -> MyResult {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;

    let mut p = Parser(s.chars().peekable());
    p.expr()?;

    emit('\n')
}

type MyResult = io::Result<()>;

fn emit(c: char) -> MyResult {
    io::stdout().write(&[c as u8]).map(|_| ())
}

struct Parser<'a>(Peekable<Chars<'a>>);

impl Parser<'_> {
    fn expr(&mut self) -> MyResult {
        self.term()?;

        loop {
            match self.peek()? {
                '+' => {
                    self._match('+')?;
                    self.term()?;
                    emit('+')?;
                }
                '-' => {
                    self._match('-')?;
                    self.term()?;
                    emit('-')?;
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn term(&mut self) -> MyResult {
        match self.peek()? {
            c if c.is_digit(10) => {
                emit(c)?;
                self._match(c)
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "expected a digit")),
        }
    }

    fn peek(&mut self) -> io::Result<char> {
        match self.0.peek() {
            Some(c) => Ok(*c),
            None => Err(Error::new(ErrorKind::UnexpectedEof, "")),
        }
    }

    fn _match(&mut self, c: char) -> MyResult {
        match self.peek()? {
            c_next if c_next == c => {
                self.0.next();
                Ok(())
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("expected the character `{}`.", c),
            )),
        }
    }
}
