use super::{EntityExprRaw, ScalarExprRaw, VecExprRaw};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f32),
    Ident(String),
    Dot,
    LParen,
    RParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '0'..='9' => {
                let mut num_str = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        num_str.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let val: f32 = num_str
                    .parse()
                    .map_err(|_| format!("Invalid number: '{}'", num_str))?;
                tokens.push(Token::Number(val));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_alphanumeric() || d == '_' {
                        ident.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            '.' => {
                chars.next();
                tokens.push(Token::Dot);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            _ => return Err(format!("Unexpected character: '{}'", c)),
        }
    }

    Ok(tokens)
}

#[derive(Debug)]
pub enum TypedExpr {
    Scalar(ScalarExprRaw),
    Vec2(VecExprRaw),
    Entity(EntityExprRaw),
}

pub enum AtomResult {
    Parsed(TypedExpr),
    Delegate,
    Unknown,
    Error(String),
}

pub trait AtomParser {
    fn try_parse_atom(&self, name: &str) -> AtomResult;
}

pub struct BlueprintAtomParser;

impl AtomParser for BlueprintAtomParser {
    fn try_parse_atom(&self, name: &str) -> AtomResult {
        match name {
            "index" => AtomResult::Parsed(TypedExpr::Scalar(ScalarExprRaw::Index)),
            "count" => AtomResult::Parsed(TypedExpr::Scalar(ScalarExprRaw::Count)),
            "caster" | "source" | "target" | "recalc" => AtomResult::Delegate,
            _ => AtomResult::Unknown,
        }
    }
}

pub struct StatAtomParser;

impl AtomParser for StatAtomParser {
    fn try_parse_atom(&self, name: &str) -> AtomResult {
        match name {
            "index" | "count" | "caster" | "source" | "target" | "recalc" => {
                AtomResult::Error(format!("'{}' is not allowed in stat formulas", name))
            }
            _ => AtomResult::Unknown,
        }
    }
}

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    atom_parser: &'a dyn AtomParser,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>, atom_parser: &'a dyn AtomParser) -> Self {
        Self { tokens, pos: 0, atom_parser }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect_token(&mut self, expected: &Token) -> Result<(), String> {
        match self.advance() {
            Some(tok) if tok == expected => Ok(()),
            Some(tok) => Err(format!("Expected {:?}, got {:?}", expected, tok)),
            None => Err(format!("Expected {:?}, got end of input", expected)),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            Some(tok) => Err(format!("Expected identifier, got {:?}", tok)),
            None => Err("Expected identifier, got end of input".to_string()),
        }
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<TypedExpr, String> {
        let mut lhs = self.parse_primary()?;

        loop {
            match self.peek() {
                Some(Token::Dot) => {
                    if self.pos + 1 < self.tokens.len() {
                        if let Token::Ident(field) = &self.tokens[self.pos + 1] {
                            if field == "x" || field == "y" {
                                let bp = 9;
                                if bp < min_bp {
                                    break;
                                }
                                let vec = match lhs {
                                    TypedExpr::Vec2(v) => v,
                                    _ => return Err(format!(".{} can only be used on vec2 expressions", field)),
                                };
                                let is_x = field == "x";
                                self.advance(); // dot
                                self.advance(); // x/y
                                lhs = if is_x {
                                    TypedExpr::Scalar(ScalarExprRaw::X(Box::new(vec)))
                                } else {
                                    TypedExpr::Scalar(ScalarExprRaw::Y(Box::new(vec)))
                                };
                                continue;
                            }
                        }
                    }
                    break;
                }
                Some(Token::Plus) | Some(Token::Minus) | Some(Token::Star) | Some(Token::Slash) => {
                    let op = self.peek().unwrap().clone();
                    let (l_bp, r_bp) = infix_binding_power(&op);
                    if l_bp < min_bp {
                        break;
                    }
                    self.advance();
                    let rhs = self.parse_expr(r_bp)?;
                    lhs = combine_infix(op, lhs, rhs)?;
                }
                _ => break,
            }
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<TypedExpr, String> {
        match self.peek() {
            Some(Token::Number(_)) => {
                let Token::Number(n) = self.advance().unwrap().clone() else {
                    unreachable!()
                };
                Ok(TypedExpr::Scalar(ScalarExprRaw::Literal(n)))
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_expr(7)?;
                match expr {
                    TypedExpr::Scalar(s) => Ok(TypedExpr::Scalar(ScalarExprRaw::Neg(Box::new(s)))),
                    _ => Err("Unary minus can only be applied to scalar expressions".to_string()),
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr(0)?;
                self.expect_token(&Token::RParen)?;
                Ok(expr)
            }
            Some(Token::Ident(_)) => {
                let Token::Ident(name) = self.advance().unwrap().clone() else {
                    unreachable!()
                };
                self.parse_ident_expr(&name)
            }
            Some(tok) => Err(format!("Unexpected token: {:?}", tok)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_ident_expr(&mut self, name: &str) -> Result<TypedExpr, String> {
        match name {
            "stat" => return self.parse_stat_call(),
            "normalize" => return self.parse_normalize(),
            "rotate" => return self.parse_rotate(),
            "lerp" => return self.parse_lerp(),
            "vec2" => return self.parse_vec2_constructor(),
            "from_angle" => return self.parse_from_angle(),
            "min" => return self.parse_min_max(true),
            "max" => return self.parse_min_max(false),
            "clamp" => return self.parse_clamp(),
            "length" => return self.parse_length(),
            "distance" => return self.parse_distance(),
            "dot" => return self.parse_dot(),
            "angle" => return self.parse_angle(),
            _ => {}
        }

        let atom_result = self.atom_parser.try_parse_atom(name);
        match atom_result {
            AtomResult::Parsed(expr) => Ok(expr),
            AtomResult::Error(msg) => Err(msg),
            AtomResult::Delegate => {
                match name {
                    "caster" | "source" | "target" => self.parse_context_access(name),
                    "recalc" => self.parse_recalc_wrapper(),
                    _ => Err(format!("Unknown delegated atom: '{}'", name)),
                }
            }
            AtomResult::Unknown => Err(format!("Unknown identifier: '{}'", name)),
        }
    }

    fn parse_context_access(&mut self, ctx_name: &str) -> Result<TypedExpr, String> {
        self.expect_token(&Token::Dot)?;
        let field = self.expect_ident()?;

        match (ctx_name, field.as_str()) {
            ("caster", "position") => Ok(TypedExpr::Vec2(VecExprRaw::CasterPos)),
            ("caster", "entity") => Ok(TypedExpr::Entity(EntityExprRaw::CasterEntity)),
            ("source", "position") => Ok(TypedExpr::Vec2(VecExprRaw::SourcePos)),
            ("source", "direction") => Ok(TypedExpr::Vec2(VecExprRaw::SourceDir)),
            ("source", "entity") => Ok(TypedExpr::Entity(EntityExprRaw::SourceEntity)),
            ("target", "position") => Ok(TypedExpr::Vec2(VecExprRaw::TargetPos)),
            ("target", "direction") => Ok(TypedExpr::Vec2(VecExprRaw::TargetDir)),
            ("target", "entity") => Ok(TypedExpr::Entity(EntityExprRaw::TargetEntity)),
            ("caster", "direction") => Err("caster.direction is not supported".to_string()),
            _ => Err(format!("Unknown field: {}.{}", ctx_name, field)),
        }
    }

    fn parse_stat_call(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let stat_name = self.expect_ident()?;
        self.expect_token(&Token::RParen)?;
        Ok(TypedExpr::Scalar(ScalarExprRaw::Stat(stat_name)))
    }

    fn parse_normalize(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let v = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let vec = expect_vec2(v, "normalize")?;
        Ok(TypedExpr::Vec2(VecExprRaw::Normalize(Box::new(vec))))
    }

    fn parse_rotate(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let v = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let angle = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let vec = expect_vec2(v, "rotate(v, ...)")?;
        let scalar = expect_scalar(angle, "rotate(..., angle)")?;
        Ok(TypedExpr::Vec2(VecExprRaw::Rotate(
            Box::new(vec),
            Box::new(scalar),
        )))
    }

    fn parse_lerp(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let a = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let b = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let t = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let va = expect_vec2(a, "lerp(a, ...)")?;
        let vb = expect_vec2(b, "lerp(..., b, ...)")?;
        let st = expect_scalar(t, "lerp(..., ..., t)")?;
        Ok(TypedExpr::Vec2(VecExprRaw::Lerp(
            Box::new(va),
            Box::new(vb),
            Box::new(st),
        )))
    }

    fn parse_vec2_constructor(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let x = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let y = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let sx = expect_scalar(x, "vec2(x, ...)")?;
        let sy = expect_scalar(y, "vec2(..., y)")?;
        Ok(TypedExpr::Vec2(VecExprRaw::Vec2Expr(
            Box::new(sx),
            Box::new(sy),
        )))
    }

    fn parse_from_angle(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let a = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let scalar = expect_scalar(a, "from_angle")?;
        Ok(TypedExpr::Vec2(VecExprRaw::FromAngle(Box::new(scalar))))
    }

    fn parse_min_max(&mut self, is_min: bool) -> Result<TypedExpr, String> {
        let fname = if is_min { "min" } else { "max" };
        self.expect_token(&Token::LParen)?;
        let a = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let b = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let sa = expect_scalar(a, &format!("{}(a, ...)", fname))?;
        let sb = expect_scalar(b, &format!("{}(..., b)", fname))?;
        if is_min {
            Ok(TypedExpr::Scalar(ScalarExprRaw::Min(
                Box::new(sa),
                Box::new(sb),
            )))
        } else {
            Ok(TypedExpr::Scalar(ScalarExprRaw::Max(
                Box::new(sa),
                Box::new(sb),
            )))
        }
    }

    fn parse_clamp(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let value = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let lo = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let hi = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let sv = expect_scalar(value, "clamp(value, ...)")?;
        let slo = expect_scalar(lo, "clamp(..., min, ...)")?;
        let shi = expect_scalar(hi, "clamp(..., ..., max)")?;
        Ok(TypedExpr::Scalar(ScalarExprRaw::Clamp(
            Box::new(sv),
            Box::new(slo),
            Box::new(shi),
        )))
    }

    fn parse_length(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let v = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let vec = expect_vec2(v, "length")?;
        Ok(TypedExpr::Scalar(ScalarExprRaw::Length(Box::new(vec))))
    }

    fn parse_distance(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let a = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let b = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let va = expect_vec2(a, "distance(a, ...)")?;
        let vb = expect_vec2(b, "distance(..., b)")?;
        Ok(TypedExpr::Scalar(ScalarExprRaw::Distance(
            Box::new(va),
            Box::new(vb),
        )))
    }

    fn parse_dot(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let a = self.parse_expr(0)?;
        self.expect_token(&Token::Comma)?;
        let b = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let va = expect_vec2(a, "dot(a, ...)")?;
        let vb = expect_vec2(b, "dot(..., b)")?;
        Ok(TypedExpr::Scalar(ScalarExprRaw::Dot(
            Box::new(va),
            Box::new(vb),
        )))
    }

    fn parse_angle(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let v = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        let vec = expect_vec2(v, "angle")?;
        Ok(TypedExpr::Scalar(ScalarExprRaw::Angle(Box::new(vec))))
    }

    fn parse_recalc_wrapper(&mut self) -> Result<TypedExpr, String> {
        self.expect_token(&Token::LParen)?;
        let inner = self.parse_expr(0)?;
        self.expect_token(&Token::RParen)?;
        match inner {
            TypedExpr::Scalar(s) => Ok(TypedExpr::Scalar(ScalarExprRaw::Recalc(Box::new(s)))),
            TypedExpr::Vec2(v) => Ok(TypedExpr::Vec2(VecExprRaw::Recalc(Box::new(v)))),
            TypedExpr::Entity(e) => Ok(TypedExpr::Entity(EntityExprRaw::Recalc(Box::new(e)))),
        }
    }
}

fn infix_binding_power(op: &Token) -> (u8, u8) {
    match op {
        Token::Plus | Token::Minus => (3, 4),
        Token::Star | Token::Slash => (5, 6),
        _ => (0, 0),
    }
}

fn expect_scalar(expr: TypedExpr, context: &str) -> Result<ScalarExprRaw, String> {
    match expr {
        TypedExpr::Scalar(s) => Ok(s),
        TypedExpr::Vec2(_) => Err(format!("Expected scalar in {}, got vec2", context)),
        TypedExpr::Entity(_) => Err(format!("Expected scalar in {}, got entity", context)),
    }
}

fn expect_vec2(expr: TypedExpr, context: &str) -> Result<VecExprRaw, String> {
    match expr {
        TypedExpr::Vec2(v) => Ok(v),
        TypedExpr::Scalar(_) => Err(format!("Expected vec2 in {}, got scalar", context)),
        TypedExpr::Entity(_) => Err(format!("Expected vec2 in {}, got entity", context)),
    }
}

fn combine_infix(op: Token, lhs: TypedExpr, rhs: TypedExpr) -> Result<TypedExpr, String> {
    match op {
        Token::Plus => match (lhs, rhs) {
            (TypedExpr::Scalar(a), TypedExpr::Scalar(b)) => {
                Ok(TypedExpr::Scalar(ScalarExprRaw::Add(Box::new(a), Box::new(b))))
            }
            (TypedExpr::Vec2(a), TypedExpr::Vec2(b)) => {
                Ok(TypedExpr::Vec2(VecExprRaw::Add(Box::new(a), Box::new(b))))
            }
            _ => Err("'+' requires matching types (scalar+scalar or vec2+vec2)".to_string()),
        },
        Token::Minus => match (lhs, rhs) {
            (TypedExpr::Scalar(a), TypedExpr::Scalar(b)) => {
                Ok(TypedExpr::Scalar(ScalarExprRaw::Sub(Box::new(a), Box::new(b))))
            }
            (TypedExpr::Vec2(a), TypedExpr::Vec2(b)) => {
                Ok(TypedExpr::Vec2(VecExprRaw::Sub(Box::new(a), Box::new(b))))
            }
            _ => Err("'-' requires matching types (scalar-scalar or vec2-vec2)".to_string()),
        },
        Token::Star => match (lhs, rhs) {
            (TypedExpr::Scalar(a), TypedExpr::Scalar(b)) => {
                Ok(TypedExpr::Scalar(ScalarExprRaw::Mul(Box::new(a), Box::new(b))))
            }
            (TypedExpr::Vec2(v), TypedExpr::Scalar(s)) => {
                Ok(TypedExpr::Vec2(VecExprRaw::Scale(Box::new(v), Box::new(s))))
            }
            (TypedExpr::Scalar(s), TypedExpr::Vec2(v)) => {
                Ok(TypedExpr::Vec2(VecExprRaw::Scale(Box::new(v), Box::new(s))))
            }
            _ => Err("'*' does not support these operand types".to_string()),
        },
        Token::Slash => match (lhs, rhs) {
            (TypedExpr::Scalar(a), TypedExpr::Scalar(b)) => {
                Ok(TypedExpr::Scalar(ScalarExprRaw::Div(Box::new(a), Box::new(b))))
            }
            _ => Err("'/' only supports scalar / scalar".to_string()),
        },
        _ => Err(format!("Unknown infix operator: {:?}", op)),
    }
}

pub fn parse_expr_string(input: &str) -> Result<TypedExpr, String> {
    parse_expr_string_with(input, &BlueprintAtomParser)
}

pub fn parse_expr_string_with(input: &str, atom_parser: &dyn AtomParser) -> Result<TypedExpr, String> {
    let tokens = lex(input)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut parser = Parser::new(tokens, atom_parser);
    let result = parser.parse_expr(0)?;
    if parser.pos < parser.tokens.len() {
        return Err(format!(
            "Unexpected token after expression: {:?}",
            parser.tokens[parser.pos]
        ));
    }
    Ok(result)
}
