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
            (Self::Start, '/') => Step::Continue(Some(Self::Slash)),
            (Self::Slash, '/') => Step::Continue(Some(Self::Comment)),
            (Self::BlockComment, '*') => Step::Continue(Some(Self::BlockCommentEnd)),
            (Self::Comment, _) | (Self::BlockComment, _) => Step::Continue(None),
            (Self::BlockCommentEnd, '/') => Step::Discard,
            (Self::Slash, '*') | (Self::BlockCommentEnd, _) => {
                Step::Continue(Some(Self::BlockComment))
            }

            (Self::Start, '<') => Step::Continue(Some(Self::Lt)),
            (Self::Lt, '=') => Step::Finish(MyToken::LessEqual, true),
            (Self::Lt, '>') => Step::Finish(MyToken::NotEqual, true),
            (Self::Lt, _) => Step::Finish(MyToken::Less, false),
            (Self::Start, '=') => Step::Finish(MyToken::Equal, true),
            (Self::Start, '>') => Step::Continue(Some(Self::Gt)),
            (Self::Gt, '=') => Step::Finish(MyToken::GreaterEqual, true),
            (Self::Gt, _) => Step::Finish(MyToken::Greater, false),

            (Self::Start, 'a'..='z') | (Self::Start, 'A'..='Z') => Step::Continue(Some(Self::Id)),
            (Self::Id, 'a'..='z') | (Self::Id, 'A'..='Z') | (Self::Id, '0'..='9') => {
                Step::Continue(None)
            }
            (Self::Id, _) => Step::Finish(MyToken::Ident, false),

            (Self::Start, '0'..='9') => Step::Continue(Some(Self::FloatLead)),
            (Self::FloatLead, '0'..='9') => Step::Continue(None),
            (Self::FloatLead, '.') => Step::Continue(Some(Self::FloatTrailFirst)),
            (Self::FloatTrailFirst, '0'..='9') => Step::Continue(Some(Self::FloatTrail)),
            (Self::FloatTrail, '0'..='9') => Step::Continue(None),
            (Self::FloatLead, 'E') | (Self::FloatTrail, 'E') => Step::Continue(Some(Self::FloatE)),
            (Self::FloatE, '+') | (Self::FloatE, '-') => Step::Continue(Some(Self::FloatExpFirst)),
            (Self::FloatE, '0'..='9') | (Self::FloatExpFirst, '0'..='9') => {
                Step::Continue(Some(Self::FloatExp))
            }
            (Self::FloatExp, '0'..='9') => Step::Continue(None),
            (Self::FloatLead, _) | (Self::FloatTrail, _) | (Self::FloatExp, _) => {
                Step::Finish(MyToken::Num, false)
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
