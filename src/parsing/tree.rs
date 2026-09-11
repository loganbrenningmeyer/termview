use std::collections::HashMap;
use std::f64::consts::{PI, TAU, E, GOLDEN_RATIO};

use crate::ui::PlotMode;
use super::{Parser, Tokenizer, BinaryOperator, UnaryOperator};

#[derive(Debug)]
pub enum Variable {
    X,
    Y,
    T,
}

impl Variable {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "t" => Some(Self::T),
            _ => None,
        }
    }
}


#[derive(Debug)]
pub enum Constant {
    Pi,
    Tau,
    E,
    Phi,
}

impl Constant {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "e" => Some(Self::E),
            "pi" => Some(Self::Pi),
            "tau" => Some(Self::Tau),
            "phi" => Some(Self::Phi),
            _ => None,
        }
    }
}


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
        match name.to_ascii_lowercase().as_str() {
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
    ConstantNode(Constant),
    VariableNode(String),
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
    /**
     * Tokenize & parse an expression string to build an abstract syntax tree
     */
    pub fn new(expression: &str) -> Result<TokenNode, String> {
        // Tokenize / parse expression into AST 
        let mut tokenizer = Tokenizer::new(expression);
        let tokens = tokenizer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let tree = parser.parse_expression(0)?;

        Ok(tree)
    }

    /**
     * Given a HashMap of variables and their values, evaluate the 
     * expression using the abstract syntax tree and return the output
     */
    pub fn evaluate(&self, vars: &HashMap<String, f64>) -> f64 {
        match self {
            TokenNode::NumberNode(num) => *num,

            TokenNode::VariableNode(name) => vars[name],

            TokenNode::ConstantNode(constant) => {
                return match constant {
                    Constant::Pi => PI,
                    Constant::Tau => TAU,
                    Constant::E => E,
                    Constant::Phi => GOLDEN_RATIO,
                };
            }

            TokenNode::UnaryNode(operator, child) => {
                // Evaluate child
                let child_val = child.evaluate(vars);

                // Apply unary operator
                return match operator {
                    UnaryOperator::Negate => -child_val,
                    UnaryOperator::Positive => child_val,
                };
            }

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
                };
            }

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
                };
            }
        }
    }

    /**
     * Validates that the variable IdentifierNodes are fit
     * for plotting (x for 2D, x & y for 3D, and t if animated)
     */
    pub fn validate_variables(&self, mode: PlotMode, animated: bool) -> Result<(), String> {
        match self {
            TokenNode::NumberNode(_) => Ok(()),

            TokenNode::ConstantNode(_) => Ok(()),

            TokenNode::VariableNode(name) => {
                let valid = match mode {
                    PlotMode::TwoD => name == "x" || (animated && name == "t"),
                    PlotMode::ThreeD => name == "x" || name == "y" || (animated && name == "t"),
                };

                if valid {
                    Ok(())
                } else {
                    Err(format!("Unknown variable '{}'", name))
                }
            }

            TokenNode::UnaryNode(_, child) => {
                child.validate_variables(mode, animated)
            }

            TokenNode::BinaryNode(_, left, right) => {
                left.validate_variables(mode, animated)?;
                right.validate_variables(mode, animated)?;
                Ok(())
            }

            TokenNode::CallNode(_, child) => {
                child.validate_variables(mode, animated)
            }
        }
    }
}