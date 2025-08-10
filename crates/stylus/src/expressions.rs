use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Str(String),
}

pub type Result = std::result::Result<Value, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(String);

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i:?}"),
            Value::Str(s) => write!(f, "{s:?}"),
        }
    }
}

impl Value {
    pub fn as_int(&self) -> i64 {
        match self {
            Value::Int(i) => *i,
            Value::Str(s) => s.parse::<i64>().unwrap_or_default(),
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Str(s) => s.clone(),
        }
    }

    pub fn as_bool(&self) -> bool {
        self.is_truthy()
    }

    fn from_bool(b: bool) -> Self {
        Value::Int(if b { 1 } else { 0 })
    }

    fn is_truthy(&self) -> bool {
        match self {
            Value::Int(i) => *i != 0,
            Value::Str(s) => !s.is_empty(),
        }
    }

    fn logical_not(self) -> Value {
        Value::from_bool(!self.is_truthy())
    }

    fn logical_and(self, other: Value) -> Value {
        Value::from_bool(self.is_truthy() && other.is_truthy())
    }

    fn logical_or(self, other: Value) -> Value {
        Value::from_bool(self.is_truthy() || other.is_truthy())
    }

    fn cmp(self, other: Value) -> Ordering {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.cmp(&b),
            (Value::Str(a), Value::Str(b)) => a.cmp(&b),
            (a, b) => a.as_int().cmp(&b.as_int()),
        }
    }

    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Str(a), Value::Str(b)) => Value::Str(format!("{}{}", a, b)),
            (a, b) => Value::Str(format!("{}{}", a.as_str(), b.as_str())),
        }
    }

    fn sub(self, other: Value) -> Value {
        Value::Int(self.as_int() - other.as_int())
    }

    fn mul(self, other: Value) -> Value {
        Value::Int(self.as_int() * other.as_int())
    }

    fn div(self, other: Value) -> Result {
        if other.as_int() == 0 {
            Err(Error("division by zero".to_string()))
        } else {
            Ok(Value::Int(self.as_int() / other.as_int()))
        }
    }

    fn pow_val(self, other: Value) -> Result {
        let a = self.as_int();
        let b = other.as_int();
        Ok(Value::Int(
            a.checked_pow(b.try_into().map_err(|_| Error("overflow".to_string()))?)
                .ok_or(Error("overflow".to_string()))?,
        ))
    }

    fn negate(self) -> Value {
        Value::Int(-self.as_int())
    }
}

fn unescape_string(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => output.push('\\'),
                Some('"') => output.push('"'),
                Some('\'') => output.push('\''),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
                None => output.push('\\'),
            }
        } else {
            output.push(ch);
        }
    }
    output
}

peg::parser!( pub grammar expression() for str {
    rule number() -> i64
        = n:$(['0'..='9']+) { n.parse().unwrap() }

    rule string() -> String
        = r#"""# s:$( ( r#"\""# / r#"\\"# / r#"\'"# / (!r#"""# [_]) )* ) r#"""# { unescape_string(s) }
        / r#"'"# s:$( ( r#"\""# / r#"\\"# / r#"\'"# / (!r#"'"# [_]) )* ) r#"'"# { unescape_string(s) }

    rule ident() -> String
        = i:$([ 'a'..='z' | 'A'..='Z' | '_' ][ 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]*) { i.to_string() }

    rule ws() = quiet!{[ ' ' | '\t' | '\r' | '\n' ]*}

    pub rule calculate(ctx: &HashMap<String, Value>) -> Result
        = ws() v:expr(ctx) ws() { v }

    rule expr(ctx: &HashMap<String, Value>) -> Result = precedence!{
        // Python-like precedence (low -> high): or, and, not, comparisons, +-, */, ^, atoms
        x:(@) ws() "or" ws() y:@ { Ok(x?.logical_or(y?)) }
        x:(@) ws() "and" ws() y:@ { Ok(x?.logical_and(y?)) }
              ws() "not" ws() v:@ { Ok(v?.logical_not()) }
        --
        x:(@) ws() ">=" ws() y:@ { Ok(Value::from_bool(x?.cmp(y?) >= Ordering::Equal)) }
        x:(@) ws() "<=" ws() y:@ { Ok(Value::from_bool(x?.cmp(y?) <= Ordering::Equal)) }
        x:(@) ws() "==" ws() y:@ { Ok(Value::from_bool(x?.cmp(y?) == Ordering::Equal)) }
        x:(@) ws() "!=" ws() y:@ { Ok(Value::from_bool(x?.cmp(y?) != Ordering::Equal)) }
        x:(@) ws() ">" ws() y:@ { Ok(Value::from_bool(x?.cmp(y?) == Ordering::Greater)) }
        x:(@) ws() "<" ws() y:@ { Ok(Value::from_bool(x?.cmp(y?) == Ordering::Less)) }
        --
        x:(@) ws() "+" ws() y:@ { Ok(x?.add(y?)) }
        x:(@) ws() "-" ws() y:@ { Ok(x?.sub(y?)) }
              ws() "-" ws() v:@ { Ok(v?.negate()) }
        --
        x:(@) ws() "*" ws() y:@ { Ok(x?.mul(y?)) }
        x:(@) ws() "/" ws() y:@ { x?.div(y?) }
        --
        x:@   ws() "^" ws() y:(@) { x?.pow_val(y?) }
        --
        ws() "str" ws() "(" ws() v:expr(ctx) ws() ")" { Ok(Value::Str(v?.as_str())) }
        ws() "int" ws() "(" ws() v:expr(ctx) ws() ")" { Ok(Value::Int(v?.as_int())) }
        ws() "(" ws() v:expr(ctx) ws() ")" { Ok(v?) }
        ws() s:string() { Ok(Value::Str(s)) }
        ws() n:number() { Ok(Value::Int(n)) }
        ws() "startswith" ws() "(" ws() a:expr(ctx) ws() "," ws() b:expr(ctx) ws() ")" {
            let s1 = a?.as_str();
            let s2 = b?.as_str();
            Ok(Value::from_bool(s1.starts_with(&s2)))
        }
        ws() "endswith" ws() "(" ws() a:expr(ctx) ws() "," ws() b:expr(ctx) ws() ")" {
            let s1 = a?.as_str();
            let s2 = b?.as_str();
            Ok(Value::from_bool(s1.ends_with(&s2)))
        }
        ws() "contains" ws() "(" ws() a:expr(ctx) ws() "," ws() b:expr(ctx) ws() ")" {
            let s1 = a?.as_str();
            let s2 = b?.as_str();
            Ok(Value::from_bool(s1.contains(&s2)))
        }
        ws() "length" ws() "(" ws() v:expr(ctx) ws() ")" {
            let s = v?.as_str();
            Ok(Value::Int(s.len() as i64))
        }
        ws() "true" { Ok(Value::from_bool(true)) }
        ws() "false" { Ok(Value::from_bool(false)) }
        ws() id:ident() { match ctx.get(&id) { Some(v) => Ok(v.clone()), None => Err(Error(format!("unknown identifier: {}", id))) } }
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_with_context() {
        let mut ctx: HashMap<String, Value> = HashMap::new();
        ctx.insert("a".into(), Value::Int(2));
        ctx.insert("b".into(), Value::Int(3));
        ctx.insert("s".into(), Value::Str("x".into()));

        // 2 + 3 == 5 => 1
        let v = expression::calculate("a + b == 5", &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Int(1));

        // logical and/or and prefix not (Python-like truthiness)
        let v = expression::calculate("(a and b) or not 0", &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(v, Value::Int(1));

        // string concat
        let v = expression::calculate("s + \"y\"", &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Str("xy".into()));
    }

    #[test]
    fn test_precedence_python_like() {
        let mut ctx: HashMap<String, Value> = HashMap::new();
        ctx.insert("a".into(), Value::Int(1));
        ctx.insert("b".into(), Value::Int(2));
        ctx.insert("c".into(), Value::Int(3));

        // and/or lower precedence than comparisons
        let v = expression::calculate("a == 1 and b == 2 or c == 0", &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(v, Value::Int(1)); // (a==1 and b==2) or (c==0)

        let v = expression::calculate("a == 0 and b == 2 or c == 3", &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(v, Value::Int(1)); // (a==0 and b==2) or (c==3)

        // not binds tighter than and/or but looser than comparisons
        let v = expression::calculate("not a == 1", &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Int(0)); // not (a==1)

        let v = expression::calculate("not a == 0 and b == 2", &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(v, Value::Int(1)); // (not (a==0)) and (b==2)
    }

    #[test]
    fn test_coercions() {
        let mut ctx: HashMap<String, Value> = HashMap::new();
        ctx.insert("n".into(), Value::Int(42));
        ctx.insert("t".into(), Value::Str("7".into()));

        let v = expression::calculate("str(n)", &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Str("42".into()));

        let v = expression::calculate("int(t) + 1", &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Int(8));

        let v = expression::calculate("str( int(\"5\") + 1 )", &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(v, Value::Str("6".into()));
    }

    #[test]
    fn test_string_quotes_and_escapes() {
        let ctx: HashMap<String, Value> = HashMap::new();

        let v = expression::calculate(r#"'a' + "b""#, &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(v, Value::Str("ab".into()));

        let v = expression::calculate(r#" '\"' "#, &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Str('"'.into()));

        let v = expression::calculate(r#" '\'' "#, &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Str("'".into()));

        let v = expression::calculate(r#" '\\' "#, &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Str(r"\".into()));

        let v = expression::calculate(r#" "\\" "#, &ctx).unwrap().unwrap();
        assert_eq!(v, Value::Str(r"\".into()));
    }

    #[test]
    fn test_nested_expressions() {
        let mut ctx: HashMap<String, Value> = HashMap::new();
        ctx.insert("ifDescr".into(), Value::Str("eth0".into()));
        let v = expression::calculate(
            r#"
        (startswith(ifDescr, 'eth') and not contains(ifDescr, '.'))
          or contains(ifDescr, "10G Ethernet Adapter")
          or contains(ifDescr, "2.5GbE Controller")
          "#,
            &ctx,
        )
        .unwrap()
        .unwrap();
        assert_eq!(v, Value::Int(1));
    }
}
