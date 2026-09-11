use super::{
    Function,
    BindingPower,
    BinaryOperator, 
    UnaryOperator,
    Token, 
    TokenNode,
};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { 
            tokens,
            position: 0,
        }
    }

    /**
     * 
     */
    pub fn parse_expression(&mut self, min_bp: usize) -> Result<TokenNode, String> {
        let mut left: TokenNode = self.parse_prefix()?;

        loop {
            let next: &Token = self.peek();

            // Determine if next can continue expression
            if *next == Token::RParen || *next == Token::End {
                return Ok(left);
            }

            // Otherwise, interpret next as an infix operator
            // - e.g., +, -, *, /, ^
            let operator: BinaryOperator = match next {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Caret => BinaryOperator::Power,
                _ => return Err(format!("Expected operator, got {:?}", next).into())
            };

            // Determine left/right infix operator binding power
            let bp: BindingPower = operator.binding_power();

            // Operator too weak for current parsing call
            // - e.g., 2 * 3 + 4, '+' would be too weak,
            //   leaving (2 * 3) as the left to create
            //   Add( Multiply(2, 3), 4 )
            if bp.left < min_bp {
                break;
            }

            // Operator strong enough, consume and determine
            // the right operand
            self.consume();

            let right = self.parse_expression(bp.right)?;

            // Grow left node with operator / right operand
            left = TokenNode::BinaryNode(
                operator,
                Box::new(left),
                Box::new(right),
            );
        }

        Ok(left)
    }

    /**
     * Parse the initial left prefix of
     * the expression
     * - e.g., -x, 3, (x * y), etc.
     */
    fn parse_prefix(&mut self) -> Result<TokenNode, String> {
        let token = self.consume();

        match token {
            Token::Number(val) => Ok(TokenNode::NumberNode(val)),

            Token::Identifier(name) => {
                // Function call, e.g., sin()
                if *self.peek() == Token::LParen {
                    self.consume();     // consume '('

                    // Get internal function call argument
                    let arg = self.parse_expression(0)?;

                    if *self.peek() != Token::RParen {
                        return Err("Expected ')' after function argument".into());
                    }

                    self.consume();     // consume ')'

                    // Convert function string to Function enum and handle unknown names
                    let function = Function::from_name(&name)
                        .ok_or_else(|| format!("Unknown function: '{name}'"))?;

                    Ok(TokenNode::CallNode(function, Box::new(arg)))

                // Variable, e.g., x, y, z, t
                } else {
                    Ok(TokenNode::IdentifierNode(name))
                }
            },

            Token::Minus => {
                let operator = UnaryOperator::Negate;
                let bp = operator.binding_power();

                let operand = self.parse_expression(bp)?;

                Ok(TokenNode::UnaryNode(
                    operator,
                    Box::new(operand),
                ))
            },
            Token::Plus => {
                let operator = UnaryOperator::Positive;
                let bp = operator.binding_power();

                let operand = self.parse_expression(bp)?;

                Ok(TokenNode::UnaryNode(
                    operator,
                    Box::new(operand),
                ))
            },
            Token::LParen => {
                let operand = self.parse_expression(0)?;

                // Ensure ends with RParen
                match self.peek() {
                    Token::RParen => {
                        self.consume();
                        Ok(operand)
                    }
                    _ => Err("Expected ')'".into())
                }
            },

            _ => Err("Expected expression".into())
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn consume(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        self.position += 1;
        token
    }
}