use std::io::{self, Read};

use dragon::token::{self, Lexer, State, Step, TokenResult};

fn main() -> io::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;

    Parser::new(&buf).parse();

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MyToken {
    OpenParen,
    CloseParen,
    Star,
    Pipe,
    NonSpecial,
}

#[derive(Default)]
struct MyState;

impl State for MyState {
    type Token = MyToken;
    type Error = ();

    fn handle_char(&self, c: char) -> Step<Self> {
        match (self, c) {
            (_, '(') => Step::Finish(MyToken::OpenParen, true),
            (_, ')') => Step::Finish(MyToken::CloseParen, true),
            (_, '*') => Step::Finish(MyToken::Star, true),
            (_, '|') => Step::Finish(MyToken::Pipe, true),
            (_, '\n') => Step::Discard,
            (_, _) => Step::Finish(MyToken::NonSpecial, true),
        }
    }

    fn try_finish(&self) -> Option<Self::Token> {
        None
    }
}

struct Parser<'a> {
    iter: Lexer<'a, MyState>,

    previous: Option<TokenResult<'a, MyToken, ()>>,
    current: Option<TokenResult<'a, MyToken, ()>>,

    node: usize,
}

/*
 *  regex     := term ( "|" term )*
 *  term      := atom+
 *  atom      := char "*"?
 *  char      := [A-Za-z] | "(" regex ")"
 */

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        let mut iter = token::lex(src);
        let current = iter.next();
        Self {
            iter,
            previous: None,
            current,
            node: 0,
        }
    }

    fn parse(&mut self) {
        println!("strict digraph {{");
        println!("\trankdir = LR;");

        let start = self.next_node();
        let accept = self.next_node();

        println!("\t{} [label = i, shape = circle];", start);
        println!("\t{} [label = f, shape = doublecircle];", accept);

        self.regex(start, accept);

        println!("}}");
    }

    fn regex(&mut self, start_node: usize, accept_node: usize) {
        loop {
            if self.current.is_none() {
                break;
            }

            let end = self.term(start_node);
            println!("\t{} -> {} [label = ϵ]", end, accept_node);

            if !self.r#match(MyToken::Pipe) {
                break;
            }
        }
    }

    fn term(&mut self, mut last_node: usize) -> usize {
        let first_node = self.next_node();
        println!("\t{} [label = \"\", shape = circle]", first_node);
        println!("\t{} -> {} [label = ϵ]", last_node, first_node);
        last_node = first_node;

        while let Some(end) = self.atom(last_node) {
            last_node = end;
        }

        last_node
    }

    fn atom(&mut self, last_node: usize) -> Option<usize> {
        let t_type;
        let text;

        if let Some(entry) = &self.current {
            t_type = entry.0.unwrap();
            text = entry.1;
        } else {
            return None;
        }

        match t_type {
            MyToken::OpenParen => {
                self.advance();

                let node_out = self.next_node();
                println!("\t{} [label = \"\", shape = circle];", node_out);
                self.regex(last_node, node_out);
                self.consume(MyToken::CloseParen);

                self.postfix_star(last_node, node_out);
                Some(node_out)
            }
            MyToken::NonSpecial => {
                self.advance();

                let end = self.next_node();
                let current = text.chars().next().unwrap();

                println!("\t{} [label = \"\", shape = circle];", end);
                println!("\t{} -> {} [label = {}]", last_node, end, current);

                self.postfix_star(last_node, end);
                Some(end)
            }
            _ => None,
        }
    }

    fn postfix_star(&mut self, start: usize, end: usize) {
        if self.r#match(MyToken::Star) {
            println!("\t{} -> {} [label = ϵ]", end, start);
        }
    }

    fn next_node(&mut self) -> usize {
        let old = self.node;
        self.node += 1;
        old
    }

    fn advance(&mut self) {
        self.previous = self.current.take();
        self.current = self.iter.next();
    }

    fn r#match(&mut self, wanted: MyToken) -> bool {
        if let Some((Ok(c), _)) = &self.current {
            if *c == wanted {
                self.advance();
                return true;
            }
        }

        false
    }

    fn consume(&mut self, wanted: MyToken) {
        if let Some((Ok(c), _)) = &self.current {
            if *c == wanted {
                self.advance();
                return;
            }
            panic!("Expected token {:?}, got {:?}", wanted, c);
        }

        panic!("Expected token {:?}, got EOF", wanted);
    }
}
