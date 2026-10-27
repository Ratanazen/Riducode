use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
            Value::String(_) => "String",
            Value::Nil => "Nil",
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, b))),
            (a, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (a, b) => Err(format!(
                "cannot add values of types {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            (a, b) => Err(format!(
                "cannot subtract {} from {}",
                b.type_name(),
                a.type_name()
            )),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            (a, b) => Err(format!(
                "cannot multiply values of types {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err("division by zero".into());
                }
                Ok(Value::Int(a / b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("division by zero".into());
                }
                Ok(Value::Float(a / b))
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("division by zero".into());
                }
                Ok(Value::Float(*a as f64 / b))
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err("division by zero".into());
                }
                Ok(Value::Float(a / *b as f64))
            }
            (a, b) => Err(format!(
                "cannot divide {} by {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err("modulo by zero".into());
                }
                Ok(Value::Int(a % b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("modulo by zero".into());
                }
                Ok(Value::Float(a % b))
            }
            (a, b) => Err(format!(
                "cannot calculate modulo of {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }

    pub fn negate(&self) -> Result<Value, String> {
        match self {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(n) => Ok(Value::Float(-n)),
            other => Err(format!("cannot negate value of type {}", other.type_name())),
        }
    }

    pub fn compare(&self, other: &Value) -> Result<std::cmp::Ordering, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(a.cmp(b)),
            (Value::Float(a), Value::Float(b)) => a
                .partial_cmp(b)
                .ok_or_else(|| "cannot compare NaN floating point numbers".into()),
            (Value::Int(a), Value::Float(b)) => (*a as f64)
                .partial_cmp(b)
                .ok_or_else(|| "cannot compare NaN floating point numbers".into()),
            (Value::Float(a), Value::Int(b)) => a
                .partial_cmp(&(*b as f64))
                .ok_or_else(|| "cannot compare NaN floating point numbers".into()),
            (a, b) => Err(format!(
                "cannot compare ordered values of types {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Nil => write!(f, "nil"),
        }
    }
}
