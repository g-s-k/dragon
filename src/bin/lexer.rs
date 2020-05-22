use std::io::{self, BufRead};

use dragon::token::{State, Step};

fn main() -> io::Result<()> {
    for s in io::stdin().lock().lines() {
        for token in dragon::token::lex::<MyState>(&s?) {
            println!("{:?}", token);
        }
    }
    Ok(())
}

#[derive(Debug)]
enum MyToken {
    LessEqual,
    NotEqual,
    Less,
    Equal,
    GreaterEqual,
    Greater,
    Ident,
    Num,
}

enum MyState {
    Start,           // 0, 9, and 12
    Lt,              // 1
    Gt,              // 6
    Id,              // 10
    FloatLead,       // 13
    FloatTrailFirst, // 14
    FloatTrail,      // 15
    FloatE,          // 16
    FloatExpFirst,   // 17
    FloatExp,        // 18

    /* extended to include C-style comments */
    Slash,
    Comment,
    BlockComment,
    BlockCommentEnd,
}

impl Default for MyState {
    fn default() -> Self {
        Self::Start
    }
}

impl State for MyState {
    type Token = MyToken;
    type Error = char;

    fn handle_char(&self, c: char) -> Step<Self> {
        match (self, c) {
            (Self::Start, c) if c.is_whitespace() => Step::Discard,
            (Self::Comment, '\n') => Step::Discard,
            (Self::Start, '/') => Step::ContinueWith(Self::Slash),
            (Self::Slash, '/') => Step::ContinueWith(Self::Comment),
            (Self::BlockComment, '*') => Step::ContinueWith(Self::BlockCommentEnd),
            (Self::Comment, _) | (Self::BlockComment, _) => Step::Continue,
            (Self::BlockCommentEnd, '/') => Step::Discard,
            (Self::Slash, '*') | (Self::BlockCommentEnd, _) => {
                Step::ContinueWith(Self::BlockComment)
            }

            (Self::Start, '<') => Step::ContinueWith(Self::Lt),
            (Self::Lt, '=') => Step::Done(MyToken::LessEqual, true),
            (Self::Lt, '>') => Step::Done(MyToken::NotEqual, true),
            (Self::Lt, _) => Step::Done(MyToken::Less, false),
            (Self::Start, '=') => Step::Done(MyToken::Equal, true),
            (Self::Start, '>') => Step::ContinueWith(Self::Gt),
            (Self::Gt, '=') => Step::Done(MyToken::GreaterEqual, true),
            (Self::Gt, _) => Step::Done(MyToken::Greater, false),

            (Self::Start, 'a'..='z') | (Self::Start, 'A'..='Z') => Step::ContinueWith(Self::Id),
            (Self::Id, 'a'..='z') | (Self::Id, 'A'..='Z') | (Self::Id, '0'..='9') => Step::Continue,
            (Self::Id, _) => Step::Done(MyToken::Ident, false),

            (Self::Start, '0'..='9') => Step::ContinueWith(Self::FloatLead),
            (Self::FloatLead, '0'..='9') => Step::Continue,
            (Self::FloatLead, '.') => Step::ContinueWith(Self::FloatTrailFirst),
            (Self::FloatTrailFirst, '0'..='9') => Step::ContinueWith(Self::FloatTrail),
            (Self::FloatTrail, '0'..='9') => Step::Continue,
            (Self::FloatLead, 'E') | (Self::FloatTrail, 'E') => Step::ContinueWith(Self::FloatE),
            (Self::FloatE, '+') | (Self::FloatE, '-') => Step::ContinueWith(Self::FloatExpFirst),
            (Self::FloatE, '0'..='9') | (Self::FloatExpFirst, '0'..='9') => {
                Step::ContinueWith(Self::FloatExp)
            }
            (Self::FloatExp, '0'..='9') => Step::Continue,
            (Self::FloatLead, _) | (Self::FloatTrail, _) | (Self::FloatExp, _) => {
                Step::Done(MyToken::Num, false)
            }
            (_, _) => Step::Abort(c),
        }
    }

    fn try_finish(&self) -> Option<Self::Token> {
        match self {
            Self::Lt => Some(MyToken::Less),
            Self::Gt => Some(MyToken::Greater),
            Self::Id => Some(MyToken::Ident),
            Self::FloatLead | Self::FloatTrail | Self::FloatExp => Some(MyToken::Num),
            _ => None,
        }
    }
}
