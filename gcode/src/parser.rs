// TODO: Checksums

pub use self::parser::Parser;

mod lexer {
    use arrayvec::ArrayString;
    use failure::Fail;


    #[derive(Debug, Fail)]
    pub enum LexerError {
        #[fail(display = "illegal symbol: {}", symbol)]
        IllegalSymbol {
            symbol: char,
        },

        #[fail(display = "invalid number: {}", text)]
        InvalidNumber {
            text: String,
        },
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum Token {
        BlockDelete,
        Letter(char),
        Number(f64),
        Demarcation,
    }

    pub struct Reader<I> {
        input: I,
        current: Option<char>,
    }

    impl<I> Reader<I>
        where I: Iterator<Item=char> {
        pub fn new(mut input: I) -> Self {
            let current = Self::next(&mut input);

            return Self {
                input,
                current,
            };
        }

        fn next(input: &mut I) -> Option<char> {
            let mut next = input.next();
            while let Some(c) = next {
                if c == ' ' || c == '\t' {
                    next = input.next();
                } else {
                    return Some(c);
                }
            }

            return None;
        }

        pub fn current(&self) -> Option<char> { self.current }

        pub fn enhance(&mut self) -> char {
            let current = self.current.expect("Enhanced after end of input");

            self.current = Self::next(&mut self.input);

            return current;
        }
    }

    pub struct Lexer<I> {
        reader: Reader<I>,

    }

    impl<I> Lexer<I>
        where I: Iterator<Item=char> {
        pub fn new(input: I) -> Self {
            Self {
                reader: Reader::new(input),
            }
        }

        fn accept_while<P, A>(&mut self, mut predicate: P, mut acceptor: A)
            where P: FnMut(char) -> bool,
                  A: FnMut(char) {
            while let Some(c) = self.reader.current() {
                if predicate(c) {
                    acceptor(c);
                    self.reader.enhance();
                } else {
                    break;
                }
            }
        }

        fn accept_until<P, A>(&mut self, mut predicate: P, mut acceptor: A)
            where P: FnMut(char) -> bool,
                  A: FnMut(char) {
            while let Some(c) = self.reader.current() {
                if !predicate(c) {
                    acceptor(c);
                    self.reader.enhance();
                } else {
                    self.reader.enhance();
                    break;
                }
            }
        }

        pub fn next(&mut self) -> Result<Option<Token>, LexerError> {
            // Skip comments
            if self.reader.current() == Some(';') { self.accept_while(|c| c != '\n', |_| {}) };
            if self.reader.current() == Some('(') { self.accept_until(|c| c == ')', |_| {}) };

            // generate tokens
            return match self.reader.current() {
                Some('/') => self.tok_block_delete(),
                Some('%') => self.tok_demarcation(),

                Some(c) if c.is_ascii_alphabetic() => self.tok_letter(),

                Some('+') | Some('-') | Some('.') => self.tok_number(),
                Some(c) if c.is_numeric() => self.tok_number(),

                Some(c) => {
                    Err(LexerError::IllegalSymbol { symbol: c })
                }
                None => {
                    Ok(None)
                }
            };
        }

        fn tok_block_delete(&mut self) -> Result<Option<Token>, LexerError> {
            let c = self.reader.enhance();
            debug_assert_eq!('/', c);

            return Ok(Some(Token::BlockDelete));
        }

        fn tok_demarcation(&mut self) -> Result<Option<Token>, LexerError> {
            let c = self.reader.enhance();
            debug_assert_eq!('%', c);

            return Ok(Some(Token::Demarcation));
        }

        fn tok_letter(&mut self) -> Result<Option<Token>, LexerError> {
            let c = self.reader.enhance();
            debug_assert!(c.is_ascii_alphabetic());

            return Ok(Some(Token::Letter(c.to_ascii_uppercase())));
        }

        fn tok_number(&mut self) -> Result<Option<Token>, LexerError> {
            let mut buffer = ArrayString::<[u8; 32]>::new();

            // There can be whitespaces inside a number - just skip them
            self.accept_while(|c| c.is_numeric() || c == '+' || c == '-' || c == '.',
                              |c| buffer.push(c));

            return match buffer.parse() {
                Ok(value) => Ok(Some(Token::Number(value))),
                Err(err) => Err(LexerError::InvalidNumber { text: buffer.to_string() }),
            };
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_reader_empty() {
            let r = Reader::new("".chars());
            assert_eq!(None, r.current());
        }

        #[test]
        fn test_reader_last() {
            let mut r = Reader::new("x".chars());
            assert_eq!(Some('x'), r.current());
            assert_eq!('x', r.enhance());

            assert_eq!(None, r.current());
        }

        #[test]
        fn test_reader_many() {
            let mut r = Reader::new("satanarchaeolidealcohellish".chars());
            assert_eq!(Some('s'), r.current());
            assert_eq!('s', r.enhance());

            assert_eq!(Some('a'), r.current());
            assert_eq!('a', r.enhance());

            assert_eq!(Some('t'), r.current());
            assert_eq!('t', r.enhance());
            assert_eq!('a', r.enhance());
            assert_eq!('n', r.enhance());
        }

        #[test]
        fn test_reader_whitespaces() {
            let mut r = Reader::new("x \t y \n z".chars());
            assert_eq!('x', r.enhance());
            assert_eq!('y', r.enhance());
            assert_eq!('\n', r.enhance());
            assert_eq!('z', r.enhance());
            assert_eq!(None, r.current());
        }

        #[test]
        fn test_lex_empty() {
            let mut l = Lexer::new("".chars());
            assert_eq!(l.next().unwrap(), None);
        }

        #[test]
        fn test_lex_block_delete() {
            let mut l = Lexer::new("/".chars());
            assert_eq!(l.next().unwrap(), Some(Token::BlockDelete));
            assert_eq!(l.next().unwrap(), None);
        }

        #[test]
        fn test_lex_demarcation() {
            let mut l = Lexer::new("%".chars());
            assert_eq!(l.next().unwrap(), Some(Token::Demarcation));
            assert_eq!(l.next().unwrap(), None);
        }

        #[test]
        fn test_lex_letter() {
            let mut l = Lexer::new("G".chars());
            assert_eq!(l.next().unwrap(), Some(Token::Letter('G')));
            assert_eq!(l.next().unwrap(), None);
        }

        #[test]
        fn test_lex_number() {
            let mut l = Lexer::new("5".chars());
            assert_eq!(l.next().unwrap(), Some(Token::Number(5.0)));
            assert_eq!(l.next().unwrap(), None);

            let mut l = Lexer::new("X5 X+5 X-5 X5.0 X-5.0 X-.3 X.7 X+2. X + 4 2 . 3".chars());
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(5.0)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(5.0)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(-5.0)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(5.0)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(-5.0)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(-0.3)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(0.7)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(2.)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(42.3)));
        }

        #[test]
        fn test_lex_whitespaces() {
            let mut l = Lexer::new(" / N123 G1  ".chars());
            assert_eq!(l.next().unwrap(), Some(Token::BlockDelete));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('N')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(123.0)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('G')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(1.0)));
            assert_eq!(l.next().unwrap(), None);
        }

        #[test]
        fn test_lex_example_01() {
            // From "The NIST RS274NGC Interpreter - Version 3"
            let mut l = Lexer::new("g0x +0. 1234y 7".chars());
            assert_eq!(l.next().unwrap(), Some(Token::Letter('G')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(0.0)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('X')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(0.1234)));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('Y')));
            assert_eq!(l.next().unwrap(), Some(Token::Number(7.0)));
            assert_eq!(l.next().unwrap(), None);
        }

        #[test]
        fn test_lex_block_comment() {
            let mut l = Lexer::new("G (ignored) G".chars());
            assert_eq!(l.next().unwrap(), Some(Token::Letter('G')));
            assert_eq!(l.next().unwrap(), Some(Token::Letter('G')));
            assert_eq!(l.next().unwrap(), None);
        }

        #[test]
        fn test_lex_line_comment() {
            let mut l = Lexer::new("G ;ignored G".chars());
            assert_eq!(l.next().unwrap(), Some(Token::Letter('G')));
            assert_eq!(l.next().unwrap(), None);
        }
    }
}

mod parser {
    use failure::Fail;
    use super::lexer::{Lexer, LexerError, Token};

    #[derive(Debug, Fail)]
    pub enum ParserError {
        #[fail(display = "syntax error: {}", 0)]
        SyntaxError(LexerError),

        #[fail(display = "unexpected token: {:?}", token)]
        UnexpectedToken {
            token: Token,
        },

        #[fail(display = "missing value")]
        MissingValue,
    }

    impl From<LexerError> for ParserError {
        fn from(err: LexerError) -> Self {
            ParserError::SyntaxError(err)
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct Word {
        mnemonic: char,
        value: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Block {
        line_number: Option<f64>,
        deleted: bool,

        words: Vec<Word>,

        line: String,
    }

    impl Block {
        pub fn empty(line: &str) -> Self {
            Self {
                line_number: None,
                deleted: false,
                words: Vec::new(),
                line: line.to_owned(),
            }
        }

        pub fn is_empty(&self) -> bool {
            self.words.is_empty()
        }
    }

    pub struct Parser {}

    impl Parser {
        pub fn new() -> Self {
            Self {}
        }

        pub fn parse_all<I, S>(&mut self, input: I) -> Result<Vec<Block>, ParserError>
            where I: Iterator<Item=S>,
                  S: AsRef<str> {
            return input.map(|line| self.parse(line))
                    .collect();
        }

        pub fn parse<S>(&mut self, line: S) -> Result<Block, ParserError>
            where S: AsRef<str> {
            let line = line.as_ref().trim();

            let mut block = Block::empty(line);

            let mut lexer = Lexer::new(line.chars());
            let mut current = lexer.next()?;

            // FIXME: Implement demarcation handling

            if current == Some(Token::BlockDelete) {
                block.deleted = true;
                current = lexer.next()?;
            }

            loop {
                match current {
                    None => break,

                    Some(Token::Letter(letter)) => {
                        current = lexer.next()?;
                        match current {
                            Some(Token::Number(value)) => {
                                current = lexer.next()?;
                                if letter == 'N' {
                                    block.line_number = Some(value);
                                } else {
                                    block.words.push(Word {
                                        mnemonic: letter,
                                        value,
                                    });
                                }
                            }
                            Some(token) => {
                                return Err(ParserError::UnexpectedToken { token });
                            }
                            None => {
                                return Err(ParserError::MissingValue);
                            }
                        }
                    }

                    Some(token) => {
                        return Err(ParserError::UnexpectedToken { token });
                    }
                }
            }

            return Ok(block);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parser_empty() {
            let b = Parser::new().parse("").unwrap();
            assert!(b.is_empty());
        }

        #[test]
        fn test_parser_simple() {
            let b = Parser::new().parse("G1").unwrap();
            assert_eq!(b, Block {
                line_number: None,
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 }],
                line: "G1".to_owned(),
            });
        }

        #[test]
        fn test_parser_multiple() {
            let b = Parser::new().parse("G1 X12.34 Y-45.67").unwrap();
            assert_eq!(b, Block {
                line_number: None,
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 12.34 },
                            Word { mnemonic: 'Y', value: -45.67 }],
                line: "G1 X12.34 Y-45.67".to_owned(),
            });
        }

        #[test]
        fn test_parser_line_number() {
            let b = Parser::new().parse("G1 N9876 X12.34 Y-45.67").unwrap();
            assert_eq!(b, Block {
                line_number: Some(9876.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 12.34 },
                            Word { mnemonic: 'Y', value: -45.67 }],
                line: "G1 N9876 X12.34 Y-45.67".to_owned(),
            });
        }

        #[test]
        fn test_parser_deleted() {
            let b = Parser::new().parse("/ G1 X100").unwrap();
            assert_eq!(b, Block {
                line_number: None,
                deleted: true,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 }],
                line: "/ G1 X100".to_owned(),
            });
        }

        #[test]
        fn test_parser_multiline() {
            let b = Parser::new().parse_all("N0010 G1 X000 Y000\nN0020 G1 X100 Y000\nN0030 G1 X100 Y100\nN0040 G1 X000 Y100\nN0050 G1 X000 Y000\n".lines()).unwrap();
            let mut b = b.iter();
            assert_eq!(b.next(), Some(&Block {
                line_number: Some(10.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
                line: "N0010 G1 X000 Y000".to_owned(),
            }));
            assert_eq!(b.next(), Some(&Block {
                line_number: Some(20.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
                line: "N0020 G1 X100 Y000".to_owned(),
            }));
            assert_eq!(b.next(), Some(&Block {
                line_number: Some(30.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 },
                            Word { mnemonic: 'Y', value: 100.0 }],
                line: "N0030 G1 X100 Y100".to_owned(),
            }));
            assert_eq!(b.next(), Some(&Block {
                line_number: Some(40.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 100.0 }],
                line: "N0040 G1 X000 Y100".to_owned(),
            }));
            assert_eq!(b.next(), Some(&Block {
                line_number: Some(50.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
                line: "N0050 G1 X000 Y000".to_owned(),
            }));
            assert_eq!(b.next(), None);
        }
    }
}
