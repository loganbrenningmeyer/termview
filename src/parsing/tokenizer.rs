#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Identifier(String),

    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Caret,      // ^

    LParen,     // (
    RParen,     // )
    Comma,      // ,

    End,
}


pub struct BindingPower {
    pub left: usize,
    pub right: usize,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Positive,
    Negate,
}

impl UnaryOperator {
    pub fn binding_power(self) -> usize {
        match self {
            Self::Positive | Self::Negate => 25,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}

impl BinaryOperator {
    pub fn binding_power(self) -> BindingPower {
        match self {
            Self::Add | Self::Subtract => BindingPower {
                left: 10,
                right: 11,
            },

            Self::Multiply | Self::Divide => BindingPower {
                left: 20,
                right: 21,
            },

            Self::Power => BindingPower {
                left: 30,
                right: 30,
            },
        }
    }
}


pub struct Tokenizer<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        // Get non-whitespace characters
        let chars: Vec<char> = self.source
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        let mut tokens: Vec<Token> = Vec::new();

        while self.position < chars.len() {
            let token: Token = match chars[self.position] {
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '^' => Token::Caret,
                '(' => Token::LParen,
                ')' => Token::RParen,
                _ => {
                    // Number
                    if chars[self.position].is_numeric() {
                        let mut num_chars: Vec<char> = vec![chars[self.position]];

                        while self.position + 1 < chars.len() {
                            let next = chars[self.position + 1];

                            if next.is_numeric() || next == '.' {
                                self.position += 1;
                                num_chars.push(chars[self.position]);
                            } else {
                                break;
                            }
                        }

                        let number: f64 = num_chars
                            .iter()
                            .collect::<String>()
                            .parse()
                            .unwrap();

                        Token::Number(number)
                    }

                    // Identifier
                    else if chars[self.position].is_alphabetic() {
                        let mut iden_chars: Vec<char> = vec![chars[self.position]];

                        while self.position + 1 < chars.len() {
                            let next = chars[self.position + 1];

                            if next.is_alphabetic() {
                                self.position += 1;
                                iden_chars.push(chars[self.position]);
                            } else {
                                break;
                            }
                        }

                        let identifier: String = iden_chars
                            .iter()
                            .collect();

                        Token::Identifier(identifier)
                    }

                    // Unknown token type
                    else {
                        return Err(format!(
                            "Unexpected character '{}' at position {}",
                            chars[self.position],
                            self.position
                        ));
                    }
                },
            };

            tokens.push(token);

            self.position += 1;
        }

        tokens.push(Token::End);
        Ok(tokens)
    }
}