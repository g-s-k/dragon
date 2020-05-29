use {
    dragon::token::{self, Lexer, State, Step, TokenResult},
    std::io::{self, Read},
};

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).unwrap();
    Parser::new(&buf).parse();
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
 *  char      := [A-Za-z0-9] | "(" regex ")"
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

        let start = self.next_node(false);
        let accept = self.next_node(false);

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
        let mut prev_node = last_node;
        let first_node = self.next_node(true);
        println!("\t{} -> {} [label = ϵ]", last_node, first_node);
        last_node = first_node;

        while let Some(end) = self.atom(prev_node, last_node) {
            prev_node = last_node;
            last_node = end;
        }

        last_node
    }

    fn atom(&mut self, prev_node: usize, last_node: usize) -> Option<usize> {
        let text = if let Some(entry) = &self.current {
            entry.1
        } else {
            return None;
        };

        let end_node;
        if self.r#match(MyToken::OpenParen) {
            end_node = self.next_node(true);
            self.regex(last_node, end_node);
            self.consume(MyToken::CloseParen);
        } else if self.r#match(MyToken::NonSpecial) {
            end_node = self.next_node(true);
            let current = text.chars().next().unwrap();
            println!("\t{} -> {} [label = {}]", last_node, end_node, current);
        } else {
            return None;
        }

        if self.r#match(MyToken::Star) {
            let new_end = self.next_node(true);
            println!(
                "\t{} -> {{{}, {}}} [label = ϵ]",
                end_node, last_node, new_end
            );
            println!("\t{} -> {} [label = ϵ]", prev_node, new_end);
            Some(new_end)
        } else {
            Some(end_node)
        }
    }

    fn next_node(&mut self, should_draw: bool) -> usize {
        let old = self.node;
        self.node += 1;

        if should_draw {
            println!("\t{} [label = \"\", shape = circle];", old);
        }

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
