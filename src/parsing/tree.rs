use std::collections::HashMap;
use super::{UnaryOperator, BinaryOperator};


#[derive(Debug)]
pub enum Function {
    Sin,
    Cos,
    Tan,
    Sqrt,
    Abs,
    Ln,
    Exp,
}

impl Function {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "sin" => Some(Self::Sin),
            "cos" => Some(Self::Cos),
            "tan" => Some(Self::Tan),
            "sqrt" => Some(Self::Sqrt),
            "abs" => Some(Self::Abs),
            "ln" => Some(Self::Ln),
            "exp" => Some(Self::Exp),
            _ => None,
        }
    }
}


#[derive(Debug)]
pub enum TokenNode {
    NumberNode(f64),
    IdentifierNode(String),
    UnaryNode(
        UnaryOperator, 
        Box<TokenNode>,
    ),
    BinaryNode(
        BinaryOperator, 
        Box<TokenNode>, 
        Box<TokenNode>,
    ),
    CallNode(
        Function,
        Box<TokenNode>,
    ),
}

impl TokenNode {
    pub fn evaluate(&self, vars: &HashMap<String, f64>) -> f64 {
        match self {
            TokenNode::NumberNode(num) => *num,
            TokenNode::IdentifierNode(name) => vars[name],

            TokenNode::UnaryNode(operator, child) => {
                // Evaluate child
                let child_val = child.evaluate(vars);

                // Apply unary operator
                return match operator {
                    UnaryOperator::Negate => -child_val,
                    UnaryOperator::Positive => child_val,
                };
            },

            TokenNode::BinaryNode(operator, left, right) => {
                // Evaluate left / right children
                let left_val = left.evaluate(vars);
                let right_val = right.evaluate(vars);

                // Apply binary operator
                return match operator {
                    BinaryOperator::Add => left_val + right_val,
                    BinaryOperator::Subtract => left_val - right_val,
                    BinaryOperator::Multiply => left_val * right_val,
                    BinaryOperator::Divide => left_val / right_val,
                    BinaryOperator::Power => left_val.powf(right_val),
                }
            },

            TokenNode::CallNode(func, child) => {
                // Evaluate child
                let child_val = child.evaluate(vars);

                // Apply function call
                return match func {
                    Function::Sin => child_val.sin(),
                    Function::Cos => child_val.cos(),
                    Function::Tan => child_val.tan(),
                    Function::Sqrt => child_val.sqrt(),
                    Function::Abs => child_val.abs(),
                    Function::Ln => child_val.ln(),
                    Function::Exp => child_val.exp(),
                }
            }
        }
    }
}