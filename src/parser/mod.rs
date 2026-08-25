use crate::ast::{
    BinOp, Block, EnumDecl, EnumVariant, Expr, FnDecl, ImplDecl, ImportDecl, ImportSpec, Item,
    MatchArm, Param, Pattern, Program, Stmt, StructDecl, StructField, TestDecl, TraitDecl,
    TraitMethodDecl, TypeAnn, UnOp,
};
use crate::error::{span_to_source, VppError, VppResult};
use crate::lexer::{Token, TokenKind};
use crate::span::Span;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    source: String,
}

impl Parser {
    pub fn new(source: impl Into<String>, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            source: source.into(),
        }
    }

    pub fn parse_program(&mut self) -> VppResult<Program> {
        let mut items = Vec::new();
        self.skip_newlines();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
            self.skip_newlines();
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> VppResult<Item> {
        let public = self.match_token(&TokenKind::Pub);
        match self.peek_kind() {
            TokenKind::Import => Ok(Item::Import(self.parse_import()?)),
            TokenKind::Struct => Ok(Item::Struct(self.parse_struct(public)?)),
            TokenKind::Enum => Ok(Item::Enum(self.parse_enum(public)?)),
            TokenKind::Trait => Ok(Item::Trait(self.parse_trait(public)?)),
            TokenKind::Impl => Ok(Item::Impl(self.parse_impl()?)),
            TokenKind::Fn => Ok(Item::Function(self.parse_function(public)?)),
            TokenKind::Test => Ok(Item::Test(self.parse_test()?)),
            _ => Ok(Item::Statement(self.parse_stmt()?)),
        }
    }

    fn parse_import(&mut self) -> VppResult<ImportDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Import)?;
        let spec = match self.peek_kind() {
            TokenKind::StringLit(_) => {
                let TokenKind::StringLit(path) = self.advance_kind() else {
                    unreachable!()
                };
                ImportSpec::FilePath(path)
            }
            TokenKind::Ident(_) => ImportSpec::Module(self.parse_module_path()?),
            other => {
                return Err(VppError::UnexpectedToken {
                    found: other.to_string(),
                    expected: "module path or string".to_string(),
                    span: span_to_source(&self.source, self.current_span()),
                    expected_help: "use `import std.io` or `import \"file.vpp\"`".to_string(),
                });
            }
        };
        Ok(ImportDecl {
            spec,
            span: start.merge(self.previous_span()),
        })
    }

    fn parse_module_path(&mut self) -> VppResult<Vec<String>> {
        let mut segments = Vec::new();
        loop {
            segments.push(self.expect_ident()?);
            if !self.match_token(&TokenKind::Dot) {
                break;
            }
        }
        Ok(segments)
    }

    fn parse_struct(&mut self, public: bool) -> VppResult<StructDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Struct)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let field_start = self.current_span();
            let field_name = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let ty = self.parse_type_ann()?;
            fields.push(StructField {
                name: field_name,
                ty,
                span: field_start.merge(self.previous_span()),
            });
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(StructDecl {
            name,
            fields,
            public,
            span: start.merge(self.previous_span()),
        })
    }

    fn parse_enum(&mut self, public: bool) -> VppResult<EnumDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Enum)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut variants = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let variant_start = self.current_span();
            let variant_name = self.expect_ident()?;
            let mut payload = Vec::new();
            if self.match_token(&TokenKind::LParen) {
                if !self.check(&TokenKind::RParen) {
                    loop {
                        payload.push(self.parse_type_ann()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RParen)?;
            }
            variants.push(EnumVariant {
                name: variant_name,
                payload,
                span: variant_start.merge(self.previous_span()),
            });
            if self.match_token(&TokenKind::Comma) {
                self.skip_newlines();
            } else {
                self.skip_newlines();
            }
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(EnumDecl {
            name,
            variants,
            public,
            span: start.merge(self.previous_span()),
        })
    }

    fn parse_trait(&mut self, public: bool) -> VppResult<TraitDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Trait)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();
        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let method_start = self.current_span();
            self.expect(&TokenKind::Fn)?;
            let method_name = self.expect_ident()?;
            self.expect(&TokenKind::LParen)?;
            let mut params = Vec::new();
            if !self.check(&TokenKind::RParen) {
                loop {
                    let param_start = self.current_span();
                    let param_name = self.expect_ident()?;
                    let ty = if param_name == "self" {
                        TypeAnn::Named("self".to_string())
                    } else {
                        self.expect(&TokenKind::Colon)?;
                        self.parse_type_ann()?
                    };
                    params.push(Param {
                        name: param_name,
                        ty,
                        span: method_start.merge(param_start),
                    });
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(&TokenKind::RParen)?;
            self.expect(&TokenKind::Arrow)?;
            let ret_type = self.parse_type_ann()?;
            methods.push(TraitMethodDecl {
                name: method_name,
                params,
                ret_type,
                span: method_start.merge(self.previous_span()),
            });
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(TraitDecl {
            name,
            methods,
            public,
            span: start.merge(self.previous_span()),
        })
    }

    fn parse_impl(&mut self) -> VppResult<ImplDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Impl)?;
        let trait_name = self.expect_ident()?;
        self.expect(&TokenKind::For)?;
        let type_name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();
        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let mut func = self.parse_function(false)?;
            for param in &mut func.params {
                if param.name == "self" && param.ty == TypeAnn::Named("self".to_string()) {
                    param.ty = TypeAnn::Named(type_name.clone());
                }
            }
            methods.push(func);
            self.skip_newlines();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(ImplDecl {
            trait_name,
            type_name,
            methods,
            span: start.merge(self.previous_span()),
        })
    }

    fn parse_function(&mut self, public: bool) -> VppResult<FnDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&TokenKind::LParen)?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let param_start = self.current_span();
                let param_name = self.expect_ident()?;
                let ty = if param_name == "self" && !self.check(&TokenKind::Colon) {
                    TypeAnn::Named("self".to_string())
                } else {
                    self.expect(&TokenKind::Colon)?;
                    self.parse_type_ann()?
                };
                params.push(Param {
                    name: param_name,
                    ty,
                    span: start.merge(param_start),
                });
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;
        let ret_type = self.parse_type_ann()?;
        let body = self.parse_block()?;
        let body_span = body.span;

        Ok(FnDecl {
            name,
            type_params,
            params,
            ret_type,
            body,
            public,
            span: start.merge(body_span),
        })
    }

    fn parse_type_params(&mut self) -> VppResult<Vec<String>> {
        if !self.match_token(&TokenKind::LBracket) {
            return Ok(Vec::new());
        }
        let mut params = Vec::new();
        if !self.check(&TokenKind::RBracket) {
            loop {
                params.push(self.expect_ident()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RBracket)?;
        Ok(params)
    }

    fn parse_type_ann(&mut self) -> VppResult<TypeAnn> {
        match self.peek_kind() {
            TokenKind::IntType => {
                self.advance();
                Ok(TypeAnn::Int)
            }
            TokenKind::FloatType => {
                self.advance();
                Ok(TypeAnn::Float)
            }
            TokenKind::BoolType => {
                self.advance();
                Ok(TypeAnn::Bool)
            }
            TokenKind::StringType => {
                self.advance();
                Ok(TypeAnn::String)
            }
            TokenKind::Option => {
                self.advance();
                self.expect(&TokenKind::Lt)?;
                let inner = self.parse_type_ann()?;
                self.expect(&TokenKind::Gt)?;
                Ok(TypeAnn::Option(Box::new(inner)))
            }
            TokenKind::Result => {
                self.advance();
                self.expect(&TokenKind::Lt)?;
                let ok = self.parse_type_ann()?;
                self.expect(&TokenKind::Comma)?;
                let err = self.parse_type_ann()?;
                self.expect(&TokenKind::Gt)?;
                Ok(TypeAnn::Result {
                    ok: Box::new(ok),
                    err: Box::new(err),
                })
            }
            TokenKind::Ident(name) if name == "array" => {
                self.advance();
                self.expect(&TokenKind::LBracket)?;
                let inner = self.parse_type_ann()?;
                self.expect(&TokenKind::RBracket)?;
                Ok(TypeAnn::Array(Box::new(inner)))
            }
            TokenKind::Ident(name) => {
                self.advance();
                Ok(TypeAnn::Named(name))
            }
            other => Err(VppError::UnexpectedToken {
                found: other.to_string(),
                expected: "type".to_string(),
                span: span_to_source(&self.source, self.current_span()),
                expected_help: "expected a type such as `int`, `Person`, or `Option<int>`".to_string(),
            }),
        }
    }

    fn parse_block(&mut self) -> VppResult<Block> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        let end = self.previous_span();
        Ok(Block {
            stmts,
            span: start.merge(end),
        })
    }

    fn parse_test(&mut self) -> VppResult<TestDecl> {
        let start = self.current_span();
        self.expect(&TokenKind::Test)?;
        let name = match self.advance_kind() {
            TokenKind::StringLit(name) => name,
            other => {
                return Err(VppError::UnexpectedToken {
                    expected: "test name string".to_string(),
                    found: other.to_string(),
                    expected_help: "test blocks require a string name, e.g. test \"my test\" { ... }".to_string(),
                    span: span_to_source(&self.source, self.previous_span()),
                })
            }
        };
        let body = self.parse_block()?;
        let body_span = body.span;
        Ok(TestDecl {
            name,
            body,
            span: start.merge(body_span),
        })
    }

    fn parse_stmt(&mut self) -> VppResult<Stmt> {
        let start = self.current_span();

        if self.match_token(&TokenKind::Let) {
            let mutable = self.match_token(&TokenKind::Mut);
            let name = self.expect_ident()?;
            let ty = if self.match_token(&TokenKind::Colon) {
                Some(self.parse_type_ann()?)
            } else {
                None
            };
            self.expect(&TokenKind::Eq)?;
            let value = self.parse_expr()?;
            let value_span = value.span();
            return Ok(Stmt::Let {
                name,
                mutable,
                ty,
                value,
                span: start.merge(value_span),
            });
        }

        if self.match_token(&TokenKind::Match) {
            let scrutinee = self.parse_expr()?;
            self.expect(&TokenKind::LBrace)?;
            self.skip_newlines();
            let mut arms = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                arms.push(self.parse_match_arm()?);
                self.skip_newlines();
            }
            self.expect(&TokenKind::RBrace)?;
            return Ok(Stmt::Match {
                scrutinee,
                arms,
                span: start.merge(self.previous_span()),
            });
        }

        if self.match_token(&TokenKind::If) {
            let condition = self.parse_expr()?;
            let then_block = self.parse_block()?;
            let else_block = if self.match_token(&TokenKind::Else) {
                Some(self.parse_block()?)
            } else {
                None
            };
            let end = else_block
                .as_ref()
                .map(|b| b.span)
                .unwrap_or(then_block.span);
            return Ok(Stmt::If {
                condition,
                then_block,
                else_block,
                span: start.merge(end),
            });
        }

        if self.match_token(&TokenKind::While) {
            let condition = self.parse_expr()?;
            let body = self.parse_block()?;
            let body_span = body.span;
            return Ok(Stmt::While {
                condition,
                body,
                span: start.merge(body_span),
            });
        }

        if self.match_token(&TokenKind::For) {
            let var = self.expect_ident()?;
            self.expect(&TokenKind::In)?;
            let iter = self.parse_expr()?;
            let body = self.parse_block()?;
            let body_span = body.span;
            return Ok(Stmt::For {
                var,
                iter,
                body,
                span: start.merge(body_span),
            });
        }

        if self.match_token(&TokenKind::Break) {
            return Ok(Stmt::Break {
                span: start.merge(self.previous_span()),
            });
        }

        if self.match_token(&TokenKind::Continue) {
            return Ok(Stmt::Continue {
                span: start.merge(self.previous_span()),
            });
        }

        if self.match_token(&TokenKind::Return) {
            let value = if self.is_expr_start() {
                Some(self.parse_expr()?)
            } else {
                None
            };
            let end = value.as_ref().map(|v| v.span()).unwrap_or(start);
            return Ok(Stmt::Return {
                value,
                span: start.merge(end),
            });
        }

        if self.check(&TokenKind::LBrace) {
            let block = self.parse_block()?;
            return Ok(Stmt::Block(block));
        }

        let expr = self.parse_expr()?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_match_arm(&mut self) -> VppResult<MatchArm> {
        let start = self.current_span();
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::FatArrow)?;
        let body = self.parse_block()?;
        let body_span = body.span;
        Ok(MatchArm {
            pattern,
            body,
            span: start.merge(body_span),
        })
    }

    fn parse_pattern(&mut self) -> VppResult<Pattern> {
        let start = self.current_span();

        if matches!(self.peek_kind(), TokenKind::Ident(ref name) if name == "_") {
            self.advance();
            return Ok(Pattern::Wildcard { span: start });
        }

        if let TokenKind::Ident(first) = self.peek_kind() {
            let first = first.clone();
            self.advance();

            if self.match_token(&TokenKind::Dot) {
                let variant = self.expect_ident()?;
                let (bindings, end) = self.parse_pattern_bindings(start)?;
                return Ok(Pattern::Variant {
                    enum_name: Some(first),
                    variant,
                    bindings,
                    span: start.merge(end),
                });
            }

            if self.check(&TokenKind::LBrace) {
                self.expect(&TokenKind::LBrace)?;
                self.skip_newlines();
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    let field = self.expect_ident()?;
                    if self.match_token(&TokenKind::Colon) {
                        let binding = self.expect_ident()?;
                        fields.push((field, binding));
                    } else {
                        fields.push((field.clone(), field));
                    }
                    if !self.match_token(&TokenKind::Comma) {
                        self.skip_newlines();
                    }
                }
                self.expect(&TokenKind::RBrace)?;
                return Ok(Pattern::Struct {
                    struct_name: Some(first),
                    fields,
                    span: start.merge(self.previous_span()),
                });
            }

            let (bindings, end) = self.parse_pattern_bindings_after_name(start)?;
            return Ok(Pattern::Variant {
                enum_name: None,
                variant: first,
                bindings,
                span: start.merge(end),
            });
        }

        if self.is_literal_start() {
            let lit = self.parse_primary()?;
            return Ok(Pattern::Literal(lit));
        }

        Err(VppError::UnexpectedToken {
            found: self.peek_kind().to_string(),
            expected: "pattern".to_string(),
            span: span_to_source(&self.source, start),
            expected_help: "expected a pattern like `Some(x)`, `None`, or `_`".to_string(),
        })
    }

    fn parse_pattern_bindings(&mut self, start: Span) -> VppResult<(Vec<String>, Span)> {
        if self.match_token(&TokenKind::LParen) {
            let mut bindings = Vec::new();
            if !self.check(&TokenKind::RParen) {
                loop {
                    bindings.push(self.expect_ident()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(&TokenKind::RParen)?;
            Ok((bindings, self.previous_span()))
        } else {
            Ok((Vec::new(), start))
        }
    }

    fn parse_pattern_bindings_after_name(&mut self, start: Span) -> VppResult<(Vec<String>, Span)> {
        self.parse_pattern_bindings(start)
    }

    fn parse_expr(&mut self) -> VppResult<Expr> {
        if self.match_token(&TokenKind::Match) {
            let start = self.previous_span();
            let scrutinee = self.parse_expr()?;
            self.expect(&TokenKind::LBrace)?;
            self.skip_newlines();
            let mut arms = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                arms.push(self.parse_match_arm()?);
                self.skip_newlines();
            }
            self.expect(&TokenKind::RBrace)?;
            return Ok(Expr::Match {
                scrutinee: Box::new(scrutinee),
                arms,
                span: start.merge(self.previous_span()),
            });
        }
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> VppResult<Expr> {
        let expr = self.parse_or()?;

        if self.match_token(&TokenKind::Eq) {
            if let Expr::Ident { name, span } = expr {
                let value = self.parse_assignment()?;
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value.clone()),
                    span: span.merge(value.span()),
                });
            }
            return Err(VppError::UnexpectedToken {
                found: "=".to_string(),
                expected: "identifier".to_string(),
                span: span_to_source(&self.source, expr.span()),
                expected_help: "only variables can be assigned with `=`".to_string(),
            });
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> VppResult<Expr> {
        let mut expr = self.parse_and()?;
        while self.match_token(&TokenKind::OrOr) {
            let op = BinOp::Or;
            let left_span = expr.span();
            let right = self.parse_and()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right.clone()),
                span: left_span.merge(right.span()),
            };
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> VppResult<Expr> {
        let mut expr = self.parse_equality()?;
        while self.match_token(&TokenKind::AndAnd) {
            let op = BinOp::And;
            let left_span = expr.span();
            let right = self.parse_equality()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right.clone()),
                span: left_span.merge(right.span()),
            };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> VppResult<Expr> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = if self.match_token(&TokenKind::EqEq) {
                BinOp::Eq
            } else if self.match_token(&TokenKind::BangEq) {
                BinOp::NotEq
            } else {
                break;
            };
            let left_span = expr.span();
            let right = self.parse_comparison()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right.clone()),
                span: left_span.merge(right.span()),
            };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> VppResult<Expr> {
        let mut expr = self.parse_term()?;
        loop {
            let op = if self.match_token(&TokenKind::Lt) {
                BinOp::Lt
            } else if self.match_token(&TokenKind::LtEq) {
                BinOp::LtEq
            } else if self.match_token(&TokenKind::Gt) {
                BinOp::Gt
            } else if self.match_token(&TokenKind::GtEq) {
                BinOp::GtEq
            } else {
                break;
            };
            let left_span = expr.span();
            let right = self.parse_term()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right.clone()),
                span: left_span.merge(right.span()),
            };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> VppResult<Expr> {
        let mut expr = self.parse_factor()?;
        loop {
            let op = if self.match_token(&TokenKind::Plus) {
                BinOp::Add
            } else if self.match_token(&TokenKind::Minus) {
                BinOp::Sub
            } else {
                break;
            };
            let left_span = expr.span();
            let right = self.parse_factor()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right.clone()),
                span: left_span.merge(right.span()),
            };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> VppResult<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.match_token(&TokenKind::Star) {
                BinOp::Mul
            } else if self.match_token(&TokenKind::Slash) {
                BinOp::Div
            } else if self.match_token(&TokenKind::Percent) {
                BinOp::Mod
            } else {
                break;
            };
            let left_span = expr.span();
            let right = self.parse_unary()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right.clone()),
                span: left_span.merge(right.span()),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> VppResult<Expr> {
        let start = self.current_span();
        if let Some(op) = UnOp::from_token(&self.peek_kind()) {
            if matches!(op, UnOp::Not) {
                self.advance();
            } else if matches!(self.peek_kind(), TokenKind::Minus) {
                self.advance();
            }
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr.clone()),
                span: start.merge(expr.span()),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> VppResult<Expr> {
        let mut expr = self.parse_primary()?;
        let mut pending_type_args: Vec<TypeAnn> = Vec::new();

        loop {
            if self.match_token(&TokenKind::LBracket) {
                let checkpoint = self.pos;
                let mut type_args = Vec::new();
                let mut parsed_types = true;
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        match self.parse_type_ann() {
                            Ok(t) => type_args.push(t),
                            Err(_) => {
                                parsed_types = false;
                                break;
                            }
                        }
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                if parsed_types && self.match_token(&TokenKind::RBracket) && self.check(&TokenKind::LParen) {
                    pending_type_args = type_args;
                    continue;
                }
                self.pos = checkpoint;
                let index = self.parse_expr()?;
                self.expect(&TokenKind::RBracket)?;
                let end = self.previous_span();
                let target_span = expr.span();
                expr = Expr::Index {
                    target: Box::new(expr),
                    index: Box::new(index.clone()),
                    span: target_span.merge(end),
                };
            } else if self.match_token(&TokenKind::LParen) {
                if let Expr::Field { ref target, ref field, span } = expr {
                    if !matches!(target.as_ref(), Expr::Ident { .. }) {
                        let mut args = Vec::new();
                        if !self.check(&TokenKind::RParen) {
                            loop {
                                args.push(self.parse_expr()?);
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                        }
                        self.expect(&TokenKind::RParen)?;
                        expr = Expr::MethodCall {
                            receiver: target.clone(),
                            method: field.clone(),
                            args,
                            span,
                        };
                        pending_type_args.clear();
                        continue;
                    }
                }

                let name = match &expr {
                    Expr::Ident { name, .. } => name.clone(),
                    Expr::Field { target, field, .. } => {
                        if let Expr::Ident { name: enum_name, .. } = target.as_ref() {
                            format!("{enum_name}.{field}")
                        } else {
                            return Err(VppError::UnexpectedToken {
                                found: "(".to_string(),
                                expected: "identifier".to_string(),
                                span: span_to_source(&self.source, expr.span()),
                                expected_help: "only functions and enum variants can be called".to_string(),
                            });
                        }
                    }
                    _ => {
                        return Err(VppError::UnexpectedToken {
                            found: "(".to_string(),
                            expected: "identifier".to_string(),
                            span: span_to_source(&self.source, expr.span()),
                            expected_help: "only functions can be called".to_string(),
                        });
                    }
                };
                let mut args = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RParen)?;
                let end = self.previous_span();
                expr = Expr::Call {
                    name,
                    type_args: pending_type_args.clone(),
                    args,
                    span: expr.span().merge(end),
                };
                pending_type_args.clear();
            } else if self.match_token(&TokenKind::Dot) {
                let field = self.expect_ident()?;
                let end = self.previous_span();
                let target_span = expr.span();
                expr = Expr::Field {
                    target: Box::new(expr),
                    field,
                    span: target_span.merge(end),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> VppResult<Expr> {
        let start = self.current_span();

        if self.match_token(&TokenKind::True) {
            return Ok(Expr::Bool {
                value: true,
                span: start.merge(self.previous_span()),
            });
        }
        if self.match_token(&TokenKind::False) {
            return Ok(Expr::Bool {
                value: false,
                span: start.merge(self.previous_span()),
            });
        }

        if let TokenKind::Ident(name) = self.peek_kind() {
            if name
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
                && self.peek_ahead_is(TokenKind::LBrace)
            {
                let name = name.clone();
                self.advance();
                self.expect(&TokenKind::LBrace)?;
                let fields = self.parse_struct_field_list()?;
                return Ok(Expr::StructLit {
                    name: Some(name),
                    fields,
                    span: start.merge(self.previous_span()),
                });
            }
        }

        match self.advance_kind() {
            TokenKind::IntLit(value) => Ok(Expr::Int {
                value,
                span: start.merge(self.previous_span()),
            }),
            TokenKind::FloatLit(value) => Ok(Expr::Float {
                value,
                span: start.merge(self.previous_span()),
            }),
            TokenKind::StringLit(value) => Ok(Expr::String {
                value,
                span: start.merge(self.previous_span()),
            }),
            TokenKind::Ident(name) => Ok(Expr::Ident {
                name,
                span: start.merge(self.previous_span()),
            }),
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                let fields = self.parse_struct_field_list()?;
                Ok(Expr::StructLit {
                    name: None,
                    fields,
                    span: start.merge(self.previous_span()),
                })
            }
            TokenKind::LBracket => {
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elements.push(self.parse_expr()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::Array {
                    elements,
                    span: start.merge(self.previous_span()),
                })
            }
            other => Err(VppError::UnexpectedToken {
                found: other.to_string(),
                expected: "expression".to_string(),
                span: span_to_source(&self.source, start),
                expected_help: "expected a literal, identifier, or grouped expression".to_string(),
            }),
        }
        .and_then(|expr| self.maybe_parse_range(expr))
    }

    fn parse_struct_field_list(&mut self) -> VppResult<Vec<(String, Expr)>> {
        self.skip_newlines();
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let field_name = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let value = self.parse_expr()?;
            fields.push((field_name, value));
            if !self.match_token(&TokenKind::Comma) {
                self.skip_newlines();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(fields)
    }

    fn maybe_parse_range(&mut self, start_expr: Expr) -> VppResult<Expr> {
        if self.match_token(&TokenKind::DotDot) {
            let start_span = start_expr.span();
            let end = self.parse_primary()?;
            return Ok(Expr::Range {
                start: Box::new(start_expr),
                end: Box::new(end.clone()),
                span: start_span.merge(end.span()),
            });
        }
        Ok(start_expr)
    }

    fn is_expr_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::IntLit(_)
                | TokenKind::FloatLit(_)
                | TokenKind::StringLit(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Ident(_)
                | TokenKind::LParen
                | TokenKind::LBrace
                | TokenKind::LBracket
                | TokenKind::Bang
                | TokenKind::Minus
                | TokenKind::Match
        )
    }

    fn is_literal_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::IntLit(_)
                | TokenKind::FloatLit(_)
                | TokenKind::StringLit(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::Bang
                | TokenKind::Minus
        )
    }

    fn peek_ahead_is(&self, kind: TokenKind) -> bool {
        self.tokens
            .get(self.pos + 1)
            .map(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(&kind))
            .unwrap_or(false)
    }

    fn expect(&mut self, kind: &TokenKind) -> VppResult<()> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(VppError::UnexpectedToken {
                found: self.peek_kind().to_string(),
                expected: kind.to_string(),
                span: span_to_source(&self.source, self.current_span()),
                expected_help: format!("expected `{kind}`"),
            })
        }
    }

    fn expect_ident(&mut self) -> VppResult<String> {
        match self.advance_kind() {
            TokenKind::Ident(name) => Ok(name),
            other => Err(VppError::UnexpectedToken {
                found: other.to_string(),
                expected: "identifier".to_string(),
                span: span_to_source(&self.source, self.previous_span()),
                expected_help: "expected an identifier name".to_string(),
            }),
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek_kind()) == std::mem::discriminant(kind)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&TokenKind::Newline) {}
    }

    fn peek_kind(&self) -> TokenKind {
        self.tokens.get(self.pos).map(|t| t.kind.clone()).unwrap_or(TokenKind::Eof)
    }

    fn advance(&mut self) -> TokenKind {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous_kind()
    }

    fn advance_kind(&mut self) -> TokenKind {
        self.advance()
    }

    fn previous_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or(Span::default())
    }

    fn previous_span(&self) -> Span {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|t| t.span)
            .unwrap_or(Span::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new(source).tokenize().unwrap();
        Parser::new(source, tokens).parse_program().unwrap()
    }

    #[test]
    fn parses_struct_and_literal() {
        let program = parse(
            "struct Person { name: string age: int }\nlet p = Person { name: \"Alex\", age: 20 }",
        );
        assert!(matches!(program.items[0], Item::Struct(_)));
    }

    #[test]
    fn parses_match() {
        let program = parse("match x { Some(n) => { print(n) } None => { print(0) } }");
        assert!(matches!(program.items[0], Item::Statement(Stmt::Match { .. })));
    }
}
