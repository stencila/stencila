use common::{
    eyre::{bail, Result},
    regex::Regex,
    serde_json, tracing,
};
use node_dispatch::dispatch_validator;
use stencila_schema::{
    self, ArrayValidator, BooleanValidator, ConstantValidator, EnumValidator, IntegerValidator,
    Node, Number, NumberValidator, StringValidator, TupleValidator, ValidatorTypes,
};

/// A trait for applying different [`ValidatorTypes`] to other nodes
pub trait Validator {
    /// Check that `node` is valid against the validator
    ///
    /// Returns `Ok` if the node is valid and a descriptive, user facing `Error`
    /// if it is not.
    fn validate(&self, node: &Node) -> Result<()>;

    /// Parse a string to a `Node` that is valid against the validator
    ///
    /// Returns `Ok(node)` if the string can be parsed, an error otherwise.
    fn parse(&self, string: &str) -> Result<Node> {
        let node =
            serde_json::from_str(string).unwrap_or_else(|_| Node::String(string.to_string()));
        self.validate(&node)?;
        Ok(node)
    }

    /// Coerce the `node` to be valid against the validator
    ///
    /// Returns the coerced node.
    fn coerce(&self, node: &Node) -> Node;

    /// Get the default node for the validator
    fn default_(&self) -> Node;
}

/// [`Validator`] implementation for `ValidatorTypes`: just dispatches to variants
impl Validator for ValidatorTypes {
    fn validate(&self, node: &Node) -> Result<()> {
        dispatch_validator!(self, validate, node)
    }

    fn parse(&self, string: &str) -> Result<Node> {
        dispatch_validator!(self, parse, string)
    }

    fn coerce(&self, node: &Node) -> Node {
        dispatch_validator!(self, coerce, node)
    }

    fn default_(&self) -> Node {
        dispatch_validator!(self, default_)
    }
}

/// [`Validator`] implementation for abstract base `Validator`: does nothing
impl Validator for stencila_schema::Validator {
    fn validate(&self, _node: &Node) -> Result<()> {
        Ok(())
    }

    fn parse(&self, string: &str) -> Result<Node> {
        Ok(serde_json::from_str(string).unwrap_or_else(|_| Node::String(string.to_string())))
    }

    fn coerce(&self, node: &Node) -> Node {
        node.clone()
    }

    fn default_(&self) -> Node {
        Node::String(String::new())
    }
}

/// [`Validator`] implementation for `ConstantValidator`: checks against constant value
impl Validator for ConstantValidator {
    fn validate(&self, node: &Node) -> Result<()> {
        match *node == *self.value {
            true => Ok(()),
            false => bail!("Value is not equal to constant: {:?}", node),
        }
    }

    fn coerce(&self, _node: &Node) -> Node {
        (*self.value).clone()
    }

    fn default_(&self) -> Node {
        (*self.value).clone()
    }
}

/// [`Validator`] implementation for `EnumValidator`: checks that value is in `values`
impl Validator for EnumValidator {
    fn validate(&self, node: &Node) -> Result<()> {
        match self.values.contains(node) {
            true => Ok(()),
            false => bail!("Value is not in enumeration: {:?}", node),
        }
    }

    fn coerce(&self, node: &Node) -> Node {
        match self.values.contains(node) {
            true => node.clone(),
            false => self.default_(),
        }
    }

    fn default_(&self) -> Node {
        // Previously we used `Null` if there were no `values`. However, that is
        // problematic for serialization-deserialization (e.g. JSON) as `null` implies
        // the property is `None`.
        self.values
            .first()
            .cloned()
            .unwrap_or(Node::String(String::new()))
    }
}

/// [`Validator`] implementation for `BooleanValidator`: coerces values to a boolean
impl Validator for BooleanValidator {
    fn validate(&self, node: &Node) -> Result<()> {
        if matches!(node, Node::Boolean(..)) {
            Ok(())
        } else {
            bail!(
                "Expected a `Boolean` got a `{}` value: {}",
                node.as_ref(),
                serde_json::to_string(node).unwrap_or_default()
            )
        }
    }

    fn parse(&self, string: &str) -> Result<Node> {
        Ok(match string {
            "0" | "false" | "no" | "off" => Node::Boolean(false),
            "1" | "true" | "yes" | "on" => Node::Boolean(true),
            _ => bail!("Unable to parse string as `Boolean`: {}", string),
        })
    }

    fn coerce(&self, node: &Node) -> Node {
        Node::Boolean(match node {
            Node::Null(..) => false,
            Node::Boolean(bool) => *bool,
            Node::Number(number) => number.0 > 0f64,
            Node::String(string) => {
                !(string.to_lowercase() == "false"
                    || string == "0"
                    || string.to_lowercase() == "no"
                    || string.to_lowercase() == "off")
            }
            _ => true,
        })
    }

    fn default_(&self) -> Node {
        Node::Boolean(false)
    }
}

impl Validator for NumberValidator {
    fn validate(&self, node: &Node) -> Result<()> {
        let num = if let Node::Number(Number(num)) = node {
            *num
        } else if let Node::Integer(num) = node {
            *num as f64
        } else {
            bail!(
                "Expected a `Number` got a `{}`: {}",
                node.as_ref(),
                serde_json::to_string(node).unwrap_or_default()
            )
        };

        if let Some(Number(min)) = self.minimum {
            if num < min {
                bail!("Number is less than minimum `{}`", min)
            }
        }

        if let Some(Number(min)) = self.exclusive_minimum {
            if num <= min {
                bail!(
                    "Number is less than, or equal to, exclusive minimum `{}`",
                    min
                )
            }
        }

        if let Some(Number(max)) = self.maximum {
            if num > max {
                bail!("Number is greater than maximum `{}`", max)
            }
        }

        if let Some(Number(max)) = self.exclusive_maximum {
            if num >= max {
                bail!(
                    "Number is greater than, or equal to, exclusive maximum `{}`",
                    max
                )
            }
        }

        if let Some(Number(modulus)) = self.multiple_of {
            if Number(num % modulus) != Number(0.0) {
                bail!("Number is not a multiple of `{}`", modulus)
            }
        }

        Ok(())
    }

    fn coerce(&self, node: &Node) -> Node {
        Node::Number(Number(match node {
            Node::Null(..) => 0.,
            Node::Boolean(bool) => match bool {
                true => 1.0,
                false => 0.0,
            },
            Node::Number(number) => number.0,
            Node::String(string) => string.parse().unwrap_or_default(),
            _ => return self.default_(),
        }))
    }

    fn default_(&self) -> Node {
        let min = self.minimum.as_ref().map(|min| min.0).unwrap_or_default();
        Node::Number(Number(min))
    }
}

impl Validator for IntegerValidator {
    fn validate(&self, node: &Node) -> Result<()> {
        if matches!(node, Node::Integer(..)) {
            NumberValidator {
                minimum: self.minimum,
                exclusive_minimum: self.exclusive_minimum,
                maximum: self.maximum,
                exclusive_maximum: self.exclusive_maximum,
                multiple_of: self.multiple_of,
                ..Default::default()
            }
            .validate(node)
        } else {
            bail!(
                "Expected an `Integer` got a `{}`: {}",
                node.as_ref(),
                serde_json::to_string(node).unwrap_or_default()
            )
        }
    }

    fn coerce(&self, node: &Node) -> Node {
        Node::Integer(match node {
            Node::Null(..) => 0,
            Node::Boolean(bool) => match bool {
                true => 1,
                false => 0,
            },
            Node::Number(number) => number.0 as i64,
            Node::String(string) => string.parse().unwrap_or_default(),
            _ => return self.default_(),
        })
    }

    fn default_(&self) -> Node {
        let min = self.minimum.as_ref().map(|min| min.0).unwrap_or_default();
        Node::Integer(min as i64)
    }
}

impl Validator for StringValidator {
    fn validate(&self, node: &Node) -> Result<()> {
        let string = if let Node::String(string) = node {
            string
        } else {
            bail!(
                "Expected a `String` got a `{}`: {}",
                node.as_ref(),
                serde_json::to_string(node).unwrap_or_default()
            )
        };

        if let Some(min) = self.min_length {
            if string.len() < min as usize {
                bail!("String is shorter than minimum length `{}`", min)
            }
        }

        if let Some(max) = self.max_length {
            if string.len() > max as usize {
                bail!("String is longer than maximum length `{}`", max)
            }
        }

        if let Some(pattern) = &self.pattern {
            let regex = Regex::new(pattern)?;
            if !regex.is_match(string) {
                bail!("String does not match pattern `{}`", pattern)
            }
        }

        Ok(())
    }

    fn parse(&self, string: &str) -> Result<Node> {
        let node = Node::String(string.to_owned());
        self.validate(&node)?;
        Ok(node)
    }

    fn coerce(&self, node: &Node) -> Node {
        let mut string = match node {
            Node::String(string) => string.clone(),
            _ => serde_json::to_string(node).unwrap_or_default(),
        };

        if let Some(min) = self.min_length {
            if string.len() < min as usize {
                string = format!("{:width$}", string, width = min as usize);
            }
        }

        if let Some(max) = self.max_length {
            if string.len() > max as usize {
                string.truncate(max as usize);
            }
        }

        if let Some(pattern) = &self.pattern {
            if let Ok(regex) = Regex::new(pattern) {
                if !regex.is_match(&string) {
                    return self.default_();
                }
            }
        }

        Node::String(string)
    }

    fn default_(&self) -> Node {
        Node::String(if let Some(min) = self.min_length {
            " ".repeat(min as usize)
        } else {
            String::new()
        })
    }
}

impl Validator for ArrayValidator {
    fn validate(&self, _node: &Node) -> Result<()> {
        tracing::error!("ArrayValidator.validate not yet implemented");
        Ok(())
    }

    fn coerce(&self, _node: &Node) -> Node {
        tracing::error!("ArrayValidator.coerce not yet implemented");
        self.default_()
    }

    fn default_(&self) -> Node {
        Node::Array(Vec::new())
    }
}

impl Validator for TupleValidator {
    fn validate(&self, _node: &Node) -> Result<()> {
        tracing::error!("TupleValidator.validate not yet implemented");
        Ok(())
    }

    fn coerce(&self, _node: &Node) -> Node {
        tracing::error!("TupleValidator.validate not yet implemented");
        self.default_()
    }

    fn default_(&self) -> Node {
        Node::Array(Vec::new())
    }
}
