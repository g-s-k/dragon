pub mod token {
    use std::{iter::Peekable, mem, str::CharIndices};

    pub fn lex<'src, S: State>(src: &'src str) -> Lexer<S> {
        Lexer {
            src,
            iter: src.char_indices().peekable(),
            start: 0,
            state: S::default(),
            done: false,
        }
    }

    pub struct Lexer<'src, S: State> {
        src: &'src str,
        iter: Peekable<CharIndices<'src>>,
        start: usize,
        state: S,
        done: bool,
    }

    impl<'src, S: State> Lexer<'src, S> {
        fn current_index(&mut self) -> usize {
            self.iter.peek().map_or(self.src.len(), |(i, _)| *i)
        }

        fn current_char(&mut self) -> Option<char> {
            self.iter.peek().map(|(_, c)| *c)
        }

        fn discard_lexeme(&mut self) {
            self.state = Default::default();
            self.advance();
            self.start = self.current_index();
        }

        fn advance(&mut self) -> Option<(usize, char)> {
            self.iter.next()
        }

        fn split_lexeme(&mut self) -> &'src str {
            let end = self.current_index();
            &self.src[mem::replace(&mut self.start, end)..end]
        }

        fn finish_token(
            &mut self,
            token: Result<S::Token, S::Error>,
            consume_current: bool,
        ) -> TokenResult<'src, S::Token, S::Error> {
            self.state = Default::default();

            if consume_current {
                self.advance();
            }

            (token, self.split_lexeme())
        }
    }

    impl<'src, S: State> Iterator for Lexer<'src, S> {
        type Item = TokenResult<'src, S::Token, S::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.done {
                return None;
            }

            while let Some(c) = self.current_char() {
                match self.state.handle_char(c) {
                    Step::Discard => self.discard_lexeme(),
                    Step::Continue => {
                        self.advance();
                    }
                    Step::ContinueWith(new_state) => {
                        self.state = new_state;
                        self.advance();
                    }
                    Step::Done(out, should_consume_current) => {
                        return Some(self.finish_token(Ok(out), should_consume_current));
                    }
                    Step::Abort(e) => {
                        self.done = true;
                        return Some(self.finish_token(Err(e), true));
                    }
                }
            }

            self.done = true;

            if let Some(t) = self.state.try_finish() {
                Some((Ok(t), self.split_lexeme()))
            } else {
                None
            }
        }
    }

    pub type TokenResult<'src, T, E> = (Result<T, E>, &'src str);

    pub enum Step<S: State> {
        Discard,
        Continue,
        ContinueWith(S),
        Done(S::Token, bool),
        Abort(S::Error),
    }

    pub trait State: Default {
        type Token;
        type Error;

        fn handle_char(&self, c: char) -> Step<Self>;

        fn try_finish(&self) -> Option<Self::Token>;
    }
}
