// TODO: Checksums

mod lexer {
    use arrayvec::ArrayString;

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum Token {
        BlockDelete,
        Letter(char),
        Number(f64),
        Demarcation,
        EOL,
    }

    pub struct Reader<I> {
        input: I,

        current: Option<char>,

        // TODO: Add position
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
                match c {
                    ' ' | '\t' => {
                        next = input.next();
                    }
                    c @ _ => return Some(c),
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

        pub fn next(&mut self) -> Option<Token> {
            // Skip comments
            if self.reader.current()? == ';' { self.accept_while(|c| c != '\n', |_| {}) };
            if self.reader.current()? == '(' { self.accept_until(|c| c == ')', |_| {}) };

            // generate tokens
            return match self.reader.current()? {
                '\n' => Some(self.tok_newline()),
                '/' => Some(self.tok_block_delete()),
                '%' => Some(self.tok_demarcation()),

                c if c.is_ascii_alphabetic() => Some(self.tok_letter()),

                '+' | '-' | '.' => Some(self.tok_number()),
                c if c.is_numeric() => Some(self.tok_number()),

                c @ _ => {
                    // FIXME: Error handling
                    unimplemented!("Unhandled character: {}", c)
                }
            };
        }

        fn tok_newline(&mut self) -> Token {
            let c = self.reader.enhance();
            debug_assert_eq!('\n', c);

            return Token::EOL;
        }

        fn tok_block_delete(&mut self) -> Token {
            let c = self.reader.enhance();
            debug_assert_eq!('/', c);

            return Token::BlockDelete;
        }

        fn tok_demarcation(&mut self) -> Token {
            let c = self.reader.enhance();
            debug_assert_eq!('%', c);

            return Token::Demarcation;
        }

        fn tok_letter(&mut self) -> Token {
            let c = self.reader.enhance();
            debug_assert!(c.is_ascii_alphabetic());

            return Token::Letter(c.to_ascii_uppercase());
        }

        fn tok_number(&mut self) -> Token {
            let mut buffer = ArrayString::<[u8; 32]>::new();

            // There can be whitespaces inside a number - just skip them
            self.accept_while(|c| c.is_numeric() || c == '+' || c == '-' || c == '.',
                              |c| buffer.push(c));

            let value: f64 = buffer.parse().expect("Failed to parse number"); // FIXME: Error handling

            return Token::Number(value);
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
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_newline() {
            let mut l = Lexer::new("\n".chars());
            assert_eq!(Some(Token::EOL), l.next());
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_block_delete() {
            let mut l = Lexer::new("/".chars());
            assert_eq!(Some(Token::BlockDelete), l.next());
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_demarcation() {
            let mut l = Lexer::new("%".chars());
            assert_eq!(Some(Token::Demarcation), l.next());
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_letter() {
            let mut l = Lexer::new("G".chars());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_number() {
            let mut l = Lexer::new("5".chars());
            assert_eq!(Some(Token::Number(5.0)), l.next());
            assert_eq!(None, l.next());

            let mut l = Lexer::new("X5 X+5 X-5 X5.0 X-5.0 X-.3 X.7 X+2. X + 4 2 . 3".chars());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(5.0)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(5.0)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(-5.0)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(5.0)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(-5.0)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(-0.3)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(0.7)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(2.)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(42.3)), l.next());
        }

        #[test]
        fn test_lex_whitespaces() {
            let mut l = Lexer::new(" / N123 G1  ".chars());
            assert_eq!(Some(Token::BlockDelete), l.next());
            assert_eq!(Some(Token::Letter('N')), l.next());
            assert_eq!(Some(Token::Number(123.0)), l.next());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(Some(Token::Number(1.0)), l.next());
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_multiline() {
            let mut l = Lexer::new(" G1  \n   G2  \n G3  \n   ".chars());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(Some(Token::Number(1.0)), l.next());
            assert_eq!(Some(Token::EOL), l.next());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(Some(Token::Number(2.0)), l.next());
            assert_eq!(Some(Token::EOL), l.next());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(Some(Token::Number(3.0)), l.next());
            assert_eq!(Some(Token::EOL), l.next());
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_example_01() {
            // From "The NIST RS274NGC Interpreter - Version 3"
            let mut l = Lexer::new("g0x +0. 1234y 7".chars());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(Some(Token::Number(0.0)), l.next());
            assert_eq!(Some(Token::Letter('X')), l.next());
            assert_eq!(Some(Token::Number(0.1234)), l.next());
            assert_eq!(Some(Token::Letter('Y')), l.next());
            assert_eq!(Some(Token::Number(7.0)), l.next());
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_block_comment() {
            let mut l = Lexer::new("G (ignored) G".chars());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(None, l.next());
        }

        #[test]
        fn test_lex_line_comment() {
            let mut l = Lexer::new("G ;ignored G\nG".chars());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(Some(Token::EOL), l.next());
            assert_eq!(Some(Token::Letter('G')), l.next());
            assert_eq!(None, l.next());
        }
    }
}

mod parser {
    use super::lexer::Lexer;
    use super::lexer::Token;

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
    }

    pub struct Parser<I> {
        lexer: Lexer<I>,
        current: Option<Token>,
    }

    impl<I> Parser<I>
        where I: Iterator<Item=char> {
        pub fn new(input: I) -> Self {
            let mut lexer = Lexer::new(input);
            let current = lexer.next();

            return Self {
                lexer,
                current,
            };
        }

        fn current(&self) -> Option<Token> {
            self.current
        }

        fn enhance(&mut self) {
            self.current = self.lexer.next();
        }

        pub fn next(&mut self) -> Option<Block> {
            // Skip empty lines
            while self.current()? == Token::EOL {
                self.enhance();
            }

            // FIXME: Implement demarcation handling

            let mut block = Block {
                line_number: None,
                deleted: false,
                words: Vec::new(),
            };

            self.parse_preamble(&mut block);
            self.parse_words(&mut block);

            // Skip trailing newline
            if self.current() == Some(Token::EOL) {
                self.enhance();
            }

            return Some(block);
        }

        fn parse_preamble(&mut self, block: &mut Block) {
            if self.current() == Some(Token::BlockDelete) {
                block.deleted = true;
                self.enhance();
            }
        }

        fn parse_words(&mut self, block: &mut Block) {
            loop {
                match self.current() {
                    None | Some(Token::EOL) => break,

                    Some(Token::Letter(letter)) => {
                        self.enhance();
                        match self.current() {
                            Some(Token::Number(value)) => {
                                self.enhance();
                                if letter == 'N' {
                                    block.line_number = Some(value);
                                } else {
                                    block.words.push(Word {
                                        mnemonic: letter,
                                        value,
                                    });
                                }
                            }
                            token @ _ => {
                                // FIXME: Error handling
                                unimplemented!("{:?}", token)
                            }
                        }
                    }

                    token @ _ => {
                        // FIXME: Error handling
                        unimplemented!("{:?}", token)
                    }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parser_empty() {
            let mut p = Parser::new("".chars());
            assert_eq!(None, p.next());
        }

        #[test]
        fn test_parser_simple() {
            let mut p = Parser::new("G1".chars());
            assert_eq!(Some(Block {
                line_number: None,
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 }],
            }), p.next());
        }

        #[test]
        fn test_parser_multiple() {
            let mut p = Parser::new("G1 X12.34 Y-45.67".chars());
            assert_eq!(Some(Block {
                line_number: None,
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 12.34 },
                            Word { mnemonic: 'Y', value: -45.67 }],
            }), p.next());
        }

        #[test]
        fn test_parser_line_number() {
            let mut p = Parser::new("G1 N9876 X12.34 Y-45.67".chars());
            assert_eq!(Some(Block {
                line_number: Some(9876.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 12.34 },
                            Word { mnemonic: 'Y', value: -45.67 }],
            }), p.next());
        }

        #[test]
        fn test_parser_deleted() {
            let mut p = Parser::new("/ G1 X100".chars());
            assert_eq!(Some(Block {
                line_number: None,
                deleted: true,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 }],
            }), p.next());
        }

        #[test]
        fn test_parser_multiline() {
            let mut p = Parser::new("N0010 G1 X000 Y000\nN0020 G1 X100 Y000\nN0030 G1 X100 Y100\nN0040 G1 X000 Y100\nN0050 G1 X000 Y000\n".chars());
            assert_eq!(Some(Block {
                line_number: Some(10.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
            }), p.next());
            assert_eq!(Some(Block {
                line_number: Some(20.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
            }), p.next());
            assert_eq!(Some(Block {
                line_number: Some(30.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 100.0 },
                            Word { mnemonic: 'Y', value: 100.0 }],
            }), p.next());
            assert_eq!(Some(Block {
                line_number: Some(40.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 100.0 }],
            }), p.next());
            assert_eq!(Some(Block {
                line_number: Some(50.0),
                deleted: false,
                words: vec![Word { mnemonic: 'G', value: 1.0 },
                            Word { mnemonic: 'X', value: 000.0 },
                            Word { mnemonic: 'Y', value: 000.0 }],
            }), p.next());
        }
    }
}
