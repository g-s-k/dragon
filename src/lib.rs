pub mod token {
    //! Framework for building a lexical analyzer by simulating a deterministic finite automaton
    //! (DFA).
    //!
    //! ## Usage
    //!
    //! Create a type (most ergonomically, an enum) that will represent the inner state of your
    //! DFA. Implement the [`State`] trait for your type, then you can call the [`lex`] function to
    //! obtain an iterator over a stream of tokens recognized in an input string.
    //!
    //! [`State`]: ./trait.State.html
    //! [`lex`]: ./fn.lex.html
    //!
    //! ## Example: floating-point numbers
    //!
    //! ```
    //! # use dragon::token::*;
    //! enum MyState {
    //!     Start,
    //!     Leading,
    //!     TrailingFirst,
    //!     Trailing,
    //!     E,
    //!     ExponentFirst,
    //!     Exponent,
    //! }
    //!
    //! impl Default for MyState {
    //!     fn default() -> Self { Self::Start }
    //! }
    //!
    //! impl State for MyState {
    //!     type Token = ();
    //!     type Error = char;
    //!
    //!     fn handle_char(&self, c: char) -> Step<Self> {
    //!         match (self, c) {
    //!             (Self::Start, c) if c.is_whitespace() => Step::Discard,
    //!             (Self::Start, '0'..='9') => Step::Continue(Some(Self::Leading)),
    //!             (Self::Leading, '.') => Step::Continue(Some(Self::TrailingFirst)),
    //!             (Self::TrailingFirst, '0'..='9') => Step::Continue(Some(Self::Trailing)),
    //!             (Self::Leading, 'E') | (Self::Trailing, 'E') => Step::Continue(Some(Self::E)),
    //!             (Self::E, '+') | (Self::E, '-') => Step::Continue(Some(Self::ExponentFirst)),
    //!             (Self::E, '0'..='9')
    //!             | (Self::ExponentFirst, '0'..='9') => Step::Continue(Some(Self::Exponent)),
    //!             (Self::Leading, '0'..='9')
    //!             | (Self::Trailing, '0'..='9')
    //!             | (Self::Exponent, '0'..='9') => Step::Continue(None),
    //!             (Self::Leading, _)
    //!             | (Self::Trailing, _)
    //!             | (Self::Exponent, _) => Step::Finish((), false),
    //!             (_, _) => Step::Abort(c),
    //!         }
    //!     }
    //!
    //!     fn try_finish(&self) -> Option<Self::Token> {
    //!         match self {
    //!             Self::Leading | Self::Trailing | Self::Exponent => Some(()),
    //!             _ => None,
    //!         }
    //!     }
    //! }
    //!
    //! let mut iter = lex::<MyState>("1 2 3.0 4.44444E44 5E6 7.50 01234.5");
    //!
    //! assert_eq!(iter.next().unwrap().1, "1");
    //! assert_eq!(iter.next().unwrap().1, "2");
    //! assert_eq!(iter.next().unwrap().1, "3.0");
    //! assert_eq!(iter.next().unwrap().1, "4.44444E44");
    //! assert_eq!(iter.next().unwrap().1, "5E6");
    //! assert_eq!(iter.next().unwrap().1, "7.50");
    //! assert_eq!(iter.next().unwrap().1, "01234.5");
    //! ```

    use std::{iter::Peekable, mem, str::CharIndices};

    /// Obtain a stream of tokens from a string.
    ///
    /// The logic for decoding tokens should be specified by implementing [`State`] on the
    /// parametric type.
    ///
    /// [`State`]: ./trait.State.html
    pub fn lex<S: State>(src: &str) -> Lexer<S> {
        Lexer {
            src,
            iter: src.char_indices().peekable(),
            start: 0,
            state: S::default(),
            done: false,
        }
    }

    /// An iterator that produces tokens from a stream of `char`s.
    ///
    /// Obtain one via the [`lex`] function.
    ///
    /// [`lex`]: ./fn.lex.html
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

        fn finish_token(
            &mut self,
            token: Result<S::Token, S::Error>,
            consume_current: bool,
        ) -> TokenResult<'src, S::Token, S::Error> {
            self.state = Default::default();

            if consume_current {
                self.advance();
            }

            let end = self.current_index();
            (token, &self.src[mem::replace(&mut self.start, end)..end])
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
                    Step::Continue(None) => {
                        self.advance();
                    }
                    Step::Continue(Some(new_state)) => {
                        self.state = new_state;
                        self.advance();
                    }
                    Step::Finish(out, should_consume_current) => {
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
                Some(self.finish_token(Ok(t), false))
            } else {
                None
            }
        }
    }

    /// Returned from the `next` method on [`Lexer`](./struct.Lexer.html).
    pub type TokenResult<'a, T, E> = (Result<T, E>, &'a str);

    /// Actions to take when processing a character.
    #[non_exhaustive]
    pub enum Step<S: State> {
        /// Ignore input up to and including the current character.
        ///
        /// The lexer will be set to the default state before inspecting the next character.
        Discard,
        /// Consume another character.
        ///
        /// If the inner value is `None`, proceed in the same state. Otherwise, move into the
        /// provided state.
        Continue(Option<S>),
        /// Finish this token.
        ///
        /// The lexer will be set to the default state before inspecting the next character. The
        /// Boolean flag indicates whether the current token consumes the character currently being
        /// read (true) or if we should re-inspect this character in the next iteration (false).
        Finish(S::Token, bool),
        /// Indicate an unrecoverable error in the current token.
        ///
        /// When this action is returned to the lexer, its token stream will be interrupted - no
        /// more tokens will be returned by its `next` method.
        Abort(S::Error),
    }

    /// Internal state of a DFA representing a language.
    ///
    /// The "start" state should be specified by implementing
    /// [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html).
    pub trait State: Default {
        /// Tokens to produce from a character stream.
        type Token;
        /// Type returned when an unrecoverable error is encountered.
        type Error;

        /// Transition between automaton states, based on current state and character.
        fn handle_char(&self, c: char) -> Step<Self>;

        /// Attempt to finish a token when there is no additional input to process.
        fn try_finish(&self) -> Option<Self::Token>;
    }
}
