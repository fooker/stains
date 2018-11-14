// TODO: Checksums

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
                    Err(LexerError::IllegalSymbol {symbol: c})
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
            assert_eq!(None, l.next().unwrap());
        }

        #[test]
        fn test_lex_block_delete() {
            let mut l = Lexer::new("/".chars());
            assert_eq!(Some(Token::BlockDelete), l.next().unwrap());
            assert_eq!(None, l.next().unwrap());
        }

        #[test]
        fn test_lex_demarcation() {
            let mut l = Lexer::new("%".chars());
            assert_eq!(Some(Token::Demarcation), l.next().unwrap());
            assert_eq!(None, l.next().unwrap());
        }

        #[test]
        fn test_lex_letter() {
            let mut l = Lexer::new("G".chars());
            assert_eq!(Some(Token::Letter('G')), l.next().unwrap());
            assert_eq!(None, l.next().unwrap());
        }

        #[test]
        fn test_lex_number() {
            let mut l = Lexer::new("5".chars());
            assert_eq!(Some(Token::Number(5.0)), l.next().unwrap());
            assert_eq!(None, l.next().unwrap());

            let mut l = Lexer::new("X5 X+5 X-5 X5.0 X-5.0 X-.3 X.7 X+2. X + 4 2 . 3".chars());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(5.0)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(5.0)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(-5.0)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(5.0)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(-5.0)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(-0.3)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(0.7)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(2.)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(42.3)), l.next().unwrap());
        }

        #[test]
        fn test_lex_whitespaces() {
            let mut l = Lexer::new(" / N123 G1  ".chars());
            assert_eq!(Some(Token::BlockDelete), l.next().unwrap());
            assert_eq!(Some(Token::Letter('N')), l.next().unwrap());
            assert_eq!(Some(Token::Number(123.0)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('G')), l.next().unwrap());
            assert_eq!(Some(Token::Number(1.0)), l.next().unwrap());
            assert_eq!(None, l.next().unwrap());
        }

        #[test]
        fn test_lex_example_01() {
            // From "The NIST RS274NGC Interpreter - Version 3"
            let mut l = Lexer::new("g0x +0. 1234y 7".chars());
            assert_eq!(Some(Token::Letter('G')), l.next().unwrap());
            assert_eq!(Some(Token::Number(0.0)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('X')), l.next().unwrap());
            assert_eq!(Some(Token::Number(0.1234)), l.next().unwrap());
            assert_eq!(Some(Token::Letter('Y')), l.next().unwrap());
            assert_eq!(Some(Token::Number(7.0)), l.next().unwrap());
            assert_eq!(None, l.next().unwrap());
        }

        #[test]
        fn test_lex_block_comment() {
            let mut l = Lexer::new("G (ignored) G".chars());
            assert_eq!(Some(Token::Letter('G')), l.next().unwrap());
            assert_eq!(Some(Token::Letter('G')), l.next().unwrap());
            assert_eq!(None, l.next().unwrap());
        }

        #[test]
        fn test_lex_line_comment() {
            let mut l = Lexer::new("G ;ignored G".chars());
            assert_eq!(Some(Token::Letter('G')), l.next().unwrap());
            assert_eq!(None, l.next().unwrap());
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

    pub struct Reader<'i, I> {
        input: I,

        current: Option<&'i str>,

        // TODO: Add position
    }

    impl<'i, I> Reader<'i, I>
        where I: Iterator<Item=&'i str> + 'i {
        pub fn new(mut input: I) -> Self {
            let current = Self::next(&mut input);

            return Self {
                input,
                current,
            };
        }

        fn next(input: &mut I) -> Option<&'i str> {
            let mut next = input.next();
            while let Some(l) = next {
                let l = l.trim();
                if l.is_empty() {
                    next = input.next();
                } else {
                    return Some(l);
                }
            }

            return None;
        }

        pub fn current(&self) -> Option<&'i str> { self.current }

        pub fn enhance(&mut self) -> &'i str {
            let current = self.current.expect("Enhanced after end of input");

            self.current = Self::next(&mut self.input);

            return current;
        }
    }

    pub struct Parser<I> {
        input: I,
    }

    impl<'i, I> Parser<I>
        where I: Iterator<Item=&'i str> + 'i {
        pub fn new(mut input: I) -> Self {
            return Self {
                input,
            };
        }

        pub fn next(&mut self) -> Result<Option<Block>, ParserError> {
            let line = match self.input.next() {
                Some(line) => line,
                None => return Ok(None),
            };

            let mut lexer = Lexer::new(line.chars());

            // FIXME: Implement demarcation handling

            let mut block = Block {
                line_number: None,
                deleted: false,
                words: Vec::new(),
                line: line.to_owned(),
            };

            let mut current = lexer.next()?;

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

            return Ok(Some(block));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parser_empty() {
            let mut p = Parser::new("".lines());
            assert_eq!(None, p.next().unwrap());
        }

        #[test]
        fn test_parser_simple() {
            let mut p = Parser::new("G1".lines());
            assert_eq!(Some(Block {
                line_number: None,
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 }],
                line: "G1".to_owned(),
            }), p.next().unwrap());
        }

        #[test]
        fn test_parser_multiple() {
            let mut p = Parser::new("G1 X12.34 Y-45.67".lines());
            assert_eq!(Some(Block {
                line_number: None,
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 12.34 },
                            Word { mnemonic: 'Y', value: -45.67 }],
                line: "G1 X12.34 Y-45.67".to_owned(),
            }), p.next().unwrap());
        }

        #[test]
        fn test_parser_line_number() {
            let mut p = Parser::new("G1 N9876 X12.34 Y-45.67".lines());
            assert_eq!(Some(Block {
                line_number: Some(9876.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 12.34 },
                            Word { mnemonic: 'Y', value: -45.67 }],
                line: "G1 N9876 X12.34 Y-45.67".to_owned(),
            }), p.next().unwrap());
        }

        #[test]
        fn test_parser_deleted() {
            let mut p = Parser::new("/ G1 X100".lines());
            assert_eq!(Some(Block {
                line_number: None,
                deleted: true,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 }],
                line: "/ G1 X100".to_owned(),
            }), p.next().unwrap());
        }

        #[test]
        fn test_parser_multiline() {
            let mut p = Parser::new("N0010 G1 X000 Y000\nN0020 G1 X100 Y000\nN0030 G1 X100 Y100\nN0040 G1 X000 Y100\nN0050 G1 X000 Y000\n".lines());
            assert_eq!(Some(Block {
                line_number: Some(10.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
                line: "N0010 G1 X000 Y000".to_owned(),
            }), p.next().unwrap());
            assert_eq!(Some(Block {
                line_number: Some(20.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
                line: "N0020 G1 X100 Y000".to_owned(),
            }), p.next().unwrap());
            assert_eq!(Some(Block {
                line_number: Some(30.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 },
                            Word { mnemonic: 'Y', value: 100.0 }],
                line: "N0030 G1 X100 Y100".to_owned(),
            }), p.next().unwrap());
            assert_eq!(Some(Block {
                line_number: Some(40.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 100.0 }],
                line: "N0040 G1 X000 Y100".to_owned(),
            }), p.next().unwrap());
            assert_eq!(Some(Block {
                line_number: Some(50.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
                line: "N0050 G1 X000 Y000".to_owned(),
            }), p.next().unwrap());
        }
    }
}
