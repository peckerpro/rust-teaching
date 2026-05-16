use rt_ast::{
    expr::*,
    item::*,
    node::*,
    pattern::*,
    stmt::Block,
    ty::*,
};
use rt_common::{
    diagnostic::DiagnosticBag,
    span::{SourceFile, Span},
    token::{Token, TokenKind},
};
use rt_lexer::Lexer;
use std::{mem, sync::Arc};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peek: Option<Token>,
    pub diagnostics: DiagnosticBag,
    file: Arc<SourceFile>,
    allow_struct_literal: bool,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>, file: Arc<SourceFile>) -> Self {
        Parser {
            lexer,
            peek: None,
            diagnostics: DiagnosticBag::new(),
            file,
            allow_struct_literal: true,
        }
    }

    fn bump(&mut self) -> Option<Token> {
        mem::replace(&mut self.peek, self.lexer.next())
    }

    fn peek_tok(&mut self) -> Option<&Token> {
        if self.peek.is_none() {
            self.peek = self.lexer.next();
        }
        self.peek.as_ref()
    }

    fn at(&mut self, kind: TokenKind) -> bool {
        matches!(self.peek_tok(), Some(t) if t.kind == kind)
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind.clone()) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(kind.clone()) {
            self.bump()
        } else {
            let msg = format!("expected {}, found {}", kind, self.peek_tok().map_or("EOF", |_| "token"));
            let span = self.peek_tok().map_or(Span::DUMMY, |t| t.span);
            self.diagnostics.error(msg, span, self.file.clone());
            None
        }
    }

    fn span(&self, lo: Span) -> Span {
        match self.peek.as_ref() {
            Some(t) if t.span.lo < lo.lo => lo.to(t.span),
            Some(t) => lo.to(t.span),
            None => lo,
        }
    }

    pub fn parse_program(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        loop {
            if self.peek_tok().is_none() {
                break;
            }
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                self.bump();
            }
        }
        items
    }

    fn parse_item(&mut self) -> Option<Item> {
        self.eat(TokenKind::KwPub);

        match self.peek_tok()?.kind.clone() {
            TokenKind::KwFn => Some(self.parse_fn_item()),
            TokenKind::KwStruct => Some(self.parse_struct_item()),
            TokenKind::KwEnum => Some(self.parse_enum_item()),
            TokenKind::KwImpl => Some(self.parse_impl_item()),
            TokenKind::KwTrait => Some(self.parse_trait_item()),
            TokenKind::KwMod => Some(self.parse_mod_item()),
            TokenKind::KwUse => Some(self.parse_use_item()),
            TokenKind::KwConst => Some(self.parse_const_item()),
            TokenKind::KwStatic => Some(self.parse_static_item()),
            TokenKind::KwType => Some(self.parse_type_alias_item()),
            _ => None,
        }
    }

    fn parse_fn_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
        let generics = self.parse_generics();

        self.expect(TokenKind::LParen);
        let mut params = Vec::new();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            params.push(self.parse_fn_param());
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen);

        let ret_ty = if self.eat(TokenKind::RArrow) {
            Some(self.parse_ty())
        } else {
            None
        };

        let body = if self.at(TokenKind::Semi) {
            self.bump();
            None
        } else {
            Some(self.parse_block())
        };

        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Fn(FnItem {
            name,
            generics,
            params,
            ret_ty,
            body,
            span,
        })
    }

    fn parse_fn_param(&mut self) -> FnParam {
        let lo = self.peek_tok().unwrap().span;
        let pattern = self.parse_pattern();
        self.expect(TokenKind::Colon);
        let ty = self.parse_ty();
        FnParam {
            pattern,
            ty,
            span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
        }
    }

    fn parse_struct_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
        let generics = self.parse_generics();

        let kind = if self.eat(TokenKind::LParen) {
            let mut fields = Vec::new();
            while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                fields.push(self.parse_ty());
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RParen);
            if fields.is_empty() {
                StructKind::Unit
            } else {
                StructKind::Tuple(fields)
            }
        } else if self.at(TokenKind::Semi) {
            self.bump();
            StructKind::Unit
        } else {
            self.expect(TokenKind::LBrace);
            let mut fields = Vec::new();
            while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                let field_name = self.expect(TokenKind::Ident)
                    .map(|t| self.lexer_slice(&t)).unwrap_or_default();
                self.expect(TokenKind::Colon);
                let ty = self.parse_ty();
                fields.push(StructField {
                    name: field_name,
                    ty,
                    span: Span::DUMMY,
                });
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RBrace);
            StructKind::Named(fields)
        };

        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Struct(StructItem {
            name,
            generics,
            kind,
            span,
        })
    }

    fn parse_enum_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
        let generics = self.parse_generics();
        self.expect(TokenKind::LBrace);
        let mut variants = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let vlo = self.peek_tok().unwrap().span;
            let vname = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
            let fields = if self.eat(TokenKind::LParen) {
                let mut tys = Vec::new();
                while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                    tys.push(self.parse_ty());
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen);
                if tys.is_empty() { None } else { Some(tys) }
            } else {
                None
            };
            variants.push(EnumVariant {
                name: vname,
                fields,
                span: vlo.to(self.peek_tok().map_or(vlo, |t| t.span)),
            });
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RBrace);
        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Enum(EnumItem {
            name,
            generics,
            variants,
            span,
        })
    }

    fn parse_impl_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let generics = self.parse_generics();

            let trait_path = if self.at(TokenKind::Ident) {
            let path = self.parse_path();
            if self.at(TokenKind::KwFor) {
                self.bump();
                Some(path)
            } else {
                let ty = self.parse_ty_with_path(path);
                let items = self.parse_impl_body();
                let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
                return Item::Impl(ImplItem {
                    generics,
                    trait_path: None,
                    ty,
                    items,
                    span,
                });
            }
        } else {
            None
        };

        let ty = if trait_path.is_some() {
            self.parse_ty()
        } else {
            unreachable!()
        };

        let items = self.parse_impl_body();
        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Impl(ImplItem {
            generics,
            trait_path,
            ty,
            items,
            span,
        })
    }

    fn parse_impl_body(&mut self) -> Vec<ImplItemKind> {
        self.expect(TokenKind::LBrace);
        let mut items = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::KwFn) {
                let fn_item = self.parse_fn_item();
                if let Item::Fn(f) = fn_item {
                    items.push(ImplItemKind::Fn(f));
                }
            } else if self.at(TokenKind::KwConst) {
                let const_item = self.parse_const_item();
                if let Item::Const(c) = const_item {
                    items.push(ImplItemKind::Const(c));
                }
            } else if self.at(TokenKind::KwType) {
                let ta = self.parse_type_alias_item();
                if let Item::TypeAlias(t) = ta {
                    items.push(ImplItemKind::TypeAlias(t));
                }
            } else {
                self.bump();
            }
        }
        self.expect(TokenKind::RBrace);
        items
    }

    fn parse_trait_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
        let generics = self.parse_generics();
        self.expect(TokenKind::LBrace);
        let mut items = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::KwFn) {
                let fn_item = self.parse_fn_item();
                if let Item::Fn(f) = fn_item {
                    items.push(TraitItemKind::Fn(f));
                }
            } else if self.at(TokenKind::KwType) {
                let ta = self.parse_type_alias_item();
                if let Item::TypeAlias(t) = ta {
                    items.push(TraitItemKind::TypeAlias(t));
                }
            } else {
                self.bump();
            }
        }
        self.expect(TokenKind::RBrace);
        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Trait(TraitItem {
            name,
            generics,
            items,
            span,
        })
    }

    fn parse_mod_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();

        let items = if self.eat(TokenKind::LBrace) {
            let mut items = Vec::new();
            while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                if let Some(item) = self.parse_item() {
                    items.push(item);
                } else {
                    self.bump();
                }
            }
            self.expect(TokenKind::RBrace);
            Some(items)
        } else {
            self.expect(TokenKind::Semi);
            None
        };

        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Mod(ModItem { name, items, span })
    }

    fn parse_use_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let path = self.parse_path();
        let alias = if self.eat(TokenKind::KwAs) {
            Some(self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default())
        } else {
            None
        };
        self.expect(TokenKind::Semi);
        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Use(UseItem { path, alias, span })
    }

    fn parse_const_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
        let ty = if self.eat(TokenKind::Colon) {
            Some(self.parse_ty())
        } else {
            None
        };
        self.expect(TokenKind::Eq);
        let value = self.parse_expr();
        self.expect(TokenKind::Semi);
        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Const(ConstItem { name, ty, value, span })
    }

    fn parse_static_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let is_mut = self.eat(TokenKind::KwMut);
        let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
        self.expect(TokenKind::Colon);
        let ty = self.parse_ty();
        self.expect(TokenKind::Eq);
        let value = self.parse_expr();
        self.expect(TokenKind::Semi);
        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::Static(StaticItem { name, is_mut, ty, value, span })
    }

    fn parse_type_alias_item(&mut self) -> Item {
        let lo = self.bump().unwrap().span;
        let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
        let generics = self.parse_generics();
        self.expect(TokenKind::Eq);
        let ty = self.parse_ty();
        self.expect(TokenKind::Semi);
        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        Item::TypeAlias(TypeAliasItem { name, generics, ty, span })
    }

    fn parse_generics(&mut self) -> Option<Generics> {
        if !self.eat(TokenKind::Lt) {
            return None;
        }
        let lo = self.peek_tok().map_or(Span::DUMMY, |t| t.span);
        let mut params = Vec::new();
        while !self.at(TokenKind::Gt) && !self.at(TokenKind::Eof) {
            let name = self.expect(TokenKind::Ident).map(|t| self.lexer_slice(&t)).unwrap_or_default();
            params.push(GenericParam::Type(name));
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::Gt);
        Some(Generics {
            params,
            span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
        })
    }

    pub fn parse_block(&mut self) -> Block {
        let lo = self.peek_tok().unwrap().span;
        self.expect(TokenKind::LBrace);
        let mut stmts = Vec::new();
        let mut expr = None;

        loop {
            if self.at(TokenKind::RBrace) || self.at(TokenKind::Eof) {
                break;
            }

            if let Some(item) = self.parse_item() {
                stmts.push(Stmt::Item(item));
                continue;
            }

            if self.at(TokenKind::Semi) {
                self.bump();
                stmts.push(Stmt::Semi);
                continue;
            }

            if self.at(TokenKind::KwLet) {
                let lo = self.bump().unwrap().span;
                let pattern = self.parse_pattern();
                let ty = if self.eat(TokenKind::Colon) {
                    Some(self.parse_ty())
                } else {
                    None
                };
                let init = if self.eat(TokenKind::Eq) {
                    Some(self.parse_expr())
                } else {
                    None
                };
                let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
                if self.eat(TokenKind::Semi) {
                    stmts.push(Stmt::Let { pattern, ty, init, span });
                } else {
                    self.expect(TokenKind::Semi);
                }
                continue;
            }

            let e = self.parse_expr();
            if self.eat(TokenKind::Semi) {
                stmts.push(Stmt::Expr(e));
            } else if self.at(TokenKind::RBrace) {
                expr = Some(Box::new(e));
                break;
            } else {
                stmts.push(Stmt::Expr(e));
            }
        }

        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
        self.expect(TokenKind::RBrace);
        Block::new(stmts, expr, span)
    }

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.parse_prefix();

        loop {
            let tok = match self.peek_tok() {
                Some(t) => t.clone(),
                None => break,
            };

            let op = self.postfix_op(&tok).or_else(|| self.infix_op(&tok));

            let Some((op_kind, lbp, rbp)) = op else {
                break;
            };

            if lbp < min_bp {
                break;
            }

            match op_kind {
                OpKind::Infix(op) => {
                    self.bump();
                    let rhs = self.parse_expr_bp(rbp);
                    let span = lhs.span().to(rhs.span());
                    lhs = Expr::Binary(BinaryExpr { op, lhs: Box::new(lhs), rhs: Box::new(rhs), span });
                }
                OpKind::Assign => {
                    self.bump();
                    let rhs = self.parse_expr_bp(rbp);
                    let span = lhs.span().to(rhs.span());
                    lhs = Expr::Assign(AssignExpr { lhs: Box::new(lhs), rhs: Box::new(rhs), span });
                }
                OpKind::Range(inclusive) => {
                    self.bump();
                    if rbp == 0 {
                        let span = lhs.span();
                        lhs = Expr::Range(RangeExpr {
                            lhs: Some(Box::new(lhs)),
                            rhs: None,
                            inclusive,
                            span,
                        });
                    } else {
                        let rhs = self.parse_expr_bp(rbp);
                        let span = lhs.span().to(rhs.span());
                        lhs = Expr::Range(RangeExpr {
                            lhs: Some(Box::new(lhs)),
                            rhs: Some(Box::new(rhs)),
                            inclusive,
                            span,
                        });
                    }
                }
                OpKind::Postfix(f) => {
                    self.bump();
                    lhs = f(self, lhs);
                }
            }
        }

        lhs
    }

    fn parse_prefix(&mut self) -> Expr {
        let tok = match self.peek_tok() {
            Some(t) => t.clone(),
            None => return Expr::Underscore(Span::DUMMY),
        };

        match tok.kind {
            TokenKind::Minus => {
                self.bump();
                let expr = self.parse_expr_bp(PREC_PREFIX);
                let span = tok.span.to(expr.span());
                Expr::Unary(UnaryExpr { op: UnaryOp::Neg, expr: Box::new(expr), span })
            }
            TokenKind::Not => {
                self.bump();
                let expr = self.parse_expr_bp(PREC_PREFIX);
                let span = tok.span.to(expr.span());
                Expr::Unary(UnaryExpr { op: UnaryOp::Not, expr: Box::new(expr), span })
            }
            TokenKind::And => {
                self.bump();
                let expr = self.parse_expr_bp(PREC_PREFIX);
                let span = tok.span.to(expr.span());
                Expr::Unary(UnaryExpr { op: UnaryOp::Ref, expr: Box::new(expr), span })
            }
            TokenKind::Star => {
                self.bump();
                let expr = self.parse_expr_bp(PREC_PREFIX);
                let span = tok.span.to(expr.span());
                Expr::Unary(UnaryExpr { op: UnaryOp::Deref, expr: Box::new(expr), span })
            }
            TokenKind::AndAnd | TokenKind::EqEq | TokenKind::Ne
            | TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge
            | TokenKind::Plus | TokenKind::Slash | TokenKind::Percent
            | TokenKind::Caret | TokenKind::Shl | TokenKind::Shr
            | TokenKind::PlusEq | TokenKind::MinusEq | TokenKind::StarEq | TokenKind::SlashEq
            | TokenKind::PercentEq | TokenKind::CaretEq | TokenKind::AndEq | TokenKind::OrEq
            | TokenKind::ShlEq | TokenKind::ShrEq | TokenKind::Eq
            | TokenKind::DotDot | TokenKind::DotDotEq | TokenKind::Comma | TokenKind::Semi
            | TokenKind::Colon | TokenKind::PathSep | TokenKind::RArrow | TokenKind::FatArrow
            | TokenKind::At | TokenKind::Dollar | TokenKind::Question
            | TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket | TokenKind::Eof
            => {
                self.bump();
                self.diagnostics.error(
                    format!("unexpected token {}", tok.kind),
                    tok.span,
                    self.file.clone(),
                );
                Expr::Underscore(tok.span)
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Expr {
        let tok = self.bump().unwrap();

        match tok.kind {
            TokenKind::Integer => {
                let raw = self.lexer_slice(&tok);
                let (value, suffix) = split_suffix(&raw);
                Expr::Literal(LiteralExpr {
                    kind: LiteralKind::Integer,
                    value,
                    suffix,
                    span: tok.span,
                })
            }
            TokenKind::Float => {
                let raw = self.lexer_slice(&tok);
                let (value, suffix) = split_suffix(&raw);
                Expr::Literal(LiteralExpr {
                    kind: LiteralKind::Float,
                    value,
                    suffix,
                    span: tok.span,
                })
            }
            TokenKind::Char => Expr::Literal(LiteralExpr {
                kind: LiteralKind::Char,
                value: self.lexer_slice(&tok),
                suffix: None,
                span: tok.span,
            }),
            TokenKind::Byte => Expr::Literal(LiteralExpr {
                kind: LiteralKind::Byte,
                value: self.lexer_slice(&tok),
                suffix: None,
                span: tok.span,
            }),
            TokenKind::String => Expr::Literal(LiteralExpr {
                kind: LiteralKind::String,
                value: self.lexer_slice(&tok),
                suffix: None,
                span: tok.span,
            }),
            TokenKind::ByteString => Expr::Literal(LiteralExpr {
                kind: LiteralKind::ByteString,
                value: self.lexer_slice(&tok),
                suffix: None,
                span: tok.span,
            }),
            TokenKind::KwTrue | TokenKind::KwFalse => Expr::Literal(LiteralExpr {
                kind: LiteralKind::Bool,
                value: self.lexer_slice(&tok),
                suffix: None,
                span: tok.span,
            }),
            TokenKind::Ident | TokenKind::KwSelfLower | TokenKind::KwSelfType => {
                let ident_name = self.lexer_slice(&tok);
                let macro_call = self.eat(TokenKind::Not);

                let mut segments = vec![PathSegment {
                    name: self.lexer_slice(&tok),
                    args: None,
                }];
                let mut span = tok.span;

                while self.eat(TokenKind::PathSep) {
                    let seg_tok = self.expect(TokenKind::Ident).unwrap();
                    let mut seg = PathSegment {
                        name: self.lexer_slice(&seg_tok),
                        args: None,
                    };
                    if self.eat(TokenKind::PathSep) {
                        if let Some(t) = self.peek_tok() {
                            if t.kind == TokenKind::Lt || t.kind == TokenKind::Ident {
                                seg.args = self.parse_generic_args();
                            }
                        }
                    } else if self.eat(TokenKind::Lt) {
                        seg.args = self.parse_generic_args_inner();
                    }
                    span = span.to(seg_tok.span);
                    segments.push(seg);
                }

                let path = PathExpr { segments: segments.clone(), span };

                if macro_call && self.at(TokenKind::LParen) {
                    self.expect(TokenKind::LParen);
                    let args = self.parse_call_args();
                    let span = tok.span.to(self.peek_tok().map_or(tok.span, |t| t.span));
                    Expr::Call(CallExpr {
                        func: Box::new(Expr::Ident(IdentExpr { name: ident_name, span: tok.span })),
                        args,
                        span,
                    })
                } else if self.allow_struct_literal && self.eat(TokenKind::LBrace) {
                    let fields = self.parse_struct_literal_body();
                    self.expect(TokenKind::RBrace);
                    Expr::Struct(StructExpr { path, fields, base: None, span })
                } else if segments.len() == 1 && segments[0].args.is_none() {
                    Expr::Ident(IdentExpr {
                        name: segments.into_iter().next().unwrap().name,
                        span,
                    })
                } else {
                    Expr::Path(PathExpr { segments, span })
                }
            }
            TokenKind::LParen => {
                if self.eat(TokenKind::RParen) {
                    return Expr::Tuple(TupleExpr {
                        elements: Vec::new(),
                        span: tok.span.to(tok.span),
                    });
                }

                let expr = self.parse_expr();
                if self.eat(TokenKind::Comma) {
                    let mut elements = vec![expr];
                    while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                        elements.push(self.parse_expr());
                        if !self.eat(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen);
                    Expr::Tuple(TupleExpr {
                        elements,
                        span: tok.span.to(self.peek_tok().map_or(tok.span, |t| t.span)),
                    })
                } else {
                    self.expect(TokenKind::RParen);
                    Expr::Paren(ParenExpr {
                        expr: Box::new(expr),
                        span: tok.span.to(self.peek_tok().map_or(tok.span, |t| t.span)),
                    })
                }
            }
            TokenKind::LBrace => {
                let mut stmts = Vec::new();
                let mut expr = None;

                loop {
                    if self.at(TokenKind::RBrace) || self.at(TokenKind::Eof) {
                        break;
                    }

                    if let Some(item) = self.parse_item() {
                        stmts.push(Stmt::Item(item));
                        continue;
                    }
                    if self.at(TokenKind::Semi) {
                        self.bump();
                        stmts.push(Stmt::Semi);
                        continue;
                    }
                    if self.at(TokenKind::KwLet) {
                        let lo = self.bump().unwrap().span;
                        let pattern = self.parse_pattern();
                        let ty = if self.eat(TokenKind::Colon) { Some(self.parse_ty()) } else { None };
                        let init = if self.eat(TokenKind::Eq) { Some(self.parse_expr()) } else { None };
                        let span = lo.to(self.peek_tok().map_or(lo, |t| t.span));
                        self.expect(TokenKind::Semi);
                        stmts.push(Stmt::Let { pattern, ty, init, span });
                        continue;
                    }

                    let e = self.parse_expr();
                    if self.eat(TokenKind::Semi) {
                        stmts.push(Stmt::Expr(e));
                    } else if self.at(TokenKind::RBrace) {
                        expr = Some(Box::new(e));
                        break;
                    } else {
                        stmts.push(Stmt::Expr(e));
                    }
                }

                let span = tok.span.to(self.peek_tok().map_or(tok.span, |t| t.span));
                self.expect(TokenKind::RBrace);
                Expr::Block(Block::new(stmts, expr, span))
            }
            TokenKind::LBracket => {
                let first = self.parse_expr();
                if self.eat(TokenKind::Semi) {
                    let count = self.parse_expr();
                    self.expect(TokenKind::RBracket);
                    Expr::Array(ArrayExpr {
                        elements: vec![count],
                        span: tok.span.to(self.peek_tok().map_or(tok.span, |t| t.span)),
                    })
                } else {
                    let mut elements = vec![first];
                    while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
                        if !self.eat(TokenKind::Comma) { break; }
                        elements.push(self.parse_expr());
                    }
                    self.expect(TokenKind::RBracket);
                    Expr::Array(ArrayExpr {
                        elements,
                        span: tok.span.to(self.peek_tok().map_or(tok.span, |t| t.span)),
                    })
                }
            }
            TokenKind::KwIf => self.parse_if_expr(tok.span),
            TokenKind::KwLoop => {
                let body = self.parse_block();
                let span = tok.span.to(body.span);
                Expr::Loop(LoopExpr { body: Box::new(body), span })
            }
            TokenKind::KwWhile => {
                let condition = self.parse_expr();
                let body = self.parse_block();
                let span = tok.span.to(body.span);
                Expr::While(WhileExpr {
                    condition: Box::new(condition),
                    body: Box::new(body),
                    span,
                })
            }
            TokenKind::KwFor => {
                let pattern = self.parse_pattern();
                self.expect(TokenKind::KwIn);
                let iterable = self.parse_expr();
                let body = self.parse_block();
                let span = tok.span.to(body.span);
                Expr::For(ForExpr {
                    pattern,
                    iterable: Box::new(iterable),
                    body: Box::new(body),
                    span,
                })
            }
            TokenKind::KwMatch => {
                self.allow_struct_literal = false;
                let scrutinee = self.parse_expr();
                self.expect(TokenKind::LBrace);
                let mut arms = Vec::new();
                while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                    let pattern = self.parse_pattern();
                    let guard = if self.eat(TokenKind::KwIf) {
                        Some(Box::new(self.parse_expr()))
                    } else {
                        None
                    };
                    self.expect(TokenKind::FatArrow);
                    let body = self.parse_expr();
                    arms.push(MatchArm { pattern, guard, body });
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
                self.allow_struct_literal = true;
                self.expect(TokenKind::RBrace);
                Expr::Match(MatchExpr {
                    scrutinee: Box::new(scrutinee),
                    arms,
                    span: tok.span.to(self.peek_tok().map_or(tok.span, |t| t.span)),
                })
            }
            TokenKind::KwReturn => {
                let expr = if self.at(TokenKind::Semi) || self.at(TokenKind::RBrace) {
                    None
                } else {
                    Some(Box::new(self.parse_expr()))
                };
                Expr::Return(ReturnExpr { expr, span: tok.span })
            }
            TokenKind::KwBreak => {
                let expr = if self.at(TokenKind::Semi) || self.at(TokenKind::RBrace) {
                    None
                } else {
                    Some(Box::new(self.parse_expr()))
                };
                Expr::Break(BreakExpr { expr, span: tok.span })
            }
            TokenKind::KwContinue => Expr::Continue(tok.span),
            TokenKind::OrOr => {
                let lo = tok.span;
                let body = self.parse_expr();
                Expr::Closure(ClosureExpr {
                    params: vec![],
                    body: Box::new(body),
                    is_move: false,
                    span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                })
            }
            TokenKind::Or => {
                let lo = tok.span;
                let mut params = Vec::new();
                while !self.at(TokenKind::Or) && !self.at(TokenKind::Eof) {
                    let pattern = self.parse_pattern();
                    let ty = if self.eat(TokenKind::Colon) { Some(self.parse_ty()) } else { None };
                    params.push((pattern, ty));
                    if !self.eat(TokenKind::Comma) { break; }
                }
                self.expect(TokenKind::Or);
                let body = self.parse_expr();
                Expr::Closure(ClosureExpr {
                    params, body: Box::new(body), is_move: false,
                    span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                })
            }
            TokenKind::KwMove => {
                let lo = tok.span;
                let mut params = Vec::new();
                self.expect(TokenKind::Or);
                while !self.at(TokenKind::Or) && !self.at(TokenKind::Eof) {
                    let pattern = self.parse_pattern();
                    let ty = if self.eat(TokenKind::Colon) { Some(self.parse_ty()) } else { None };
                    params.push((pattern, ty));
                    if !self.eat(TokenKind::Comma) { break; }
                }
                self.expect(TokenKind::Or);
                let body = self.parse_expr();
                Expr::Closure(ClosureExpr {
                    params, body: Box::new(body), is_move: true,
                    span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                })
            }
            TokenKind::Underscore => Expr::Underscore(tok.span),
            _ => {
                self.diagnostics.error(
                    format!("unexpected token {}", tok.kind),
                    tok.span,
                    self.file.clone(),
                );
                Expr::Underscore(tok.span)
            }
        }
    }

    fn parse_if_expr(&mut self, lo: Span) -> Expr {
        let condition = self.parse_expr();
        let then_branch = self.parse_block();
        let else_branch = if self.eat(TokenKind::KwElse) {
            if self.at(TokenKind::KwIf) {
                Some(Box::new(self.parse_if_expr(lo)))
            } else {
                Some(Box::new(Expr::Block(self.parse_block())))
            }
        } else {
            None
        };
        Expr::If(IfExpr {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
            span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
        })
    }

    fn parse_pattern(&mut self) -> Pattern {
        match self.peek_tok().map(|t| t.kind.clone()) {
            Some(TokenKind::Ident) => {
                let tok = self.bump().unwrap();
                let name = self.lexer_slice(&tok);
                let is_mut = name == "mut";
                let is_ref = name == "ref";

                if (is_mut || is_ref) && self.at(TokenKind::Ident) {
                    let inner_tok = self.bump().unwrap();
                    let inner_name = self.lexer_slice(&inner_tok);
                    Pattern::Ident(IdentPattern {
                        name: inner_name,
                        is_mut,
                        is_ref,
                        span: tok.span.to(inner_tok.span),
                    })
                } else if name == "_" {
                    Pattern::Wildcard(tok.span)
                } else {
                    Pattern::Ident(IdentPattern {
                        name,
                        is_mut: false,
                        is_ref: false,
                        span: tok.span,
                    })
                }
            }
            Some(TokenKind::KwMut) => {
                let lo = self.bump().unwrap().span;
                let tok = self.expect(TokenKind::Ident).unwrap();
                let name = self.lexer_slice(&tok);
                Pattern::Ident(IdentPattern {
                    name,
                    is_mut: true,
                    is_ref: false,
                    span: lo.to(tok.span),
                })
            }
            Some(TokenKind::KwRef) => {
                let lo = self.bump().unwrap().span;
                let is_mut = self.eat(TokenKind::KwMut);
                let inner = self.parse_pattern();
                Pattern::Ref(RefPattern {
                    is_mut,
                    inner: Box::new(inner),
                    span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                })
            }
            Some(TokenKind::Underscore) => {
                let tok = self.bump().unwrap();
                Pattern::Wildcard(tok.span)
            }
            Some(TokenKind::Integer) | Some(TokenKind::Float)
            | Some(TokenKind::Char) | Some(TokenKind::String)
            | Some(TokenKind::KwTrue) | Some(TokenKind::KwFalse) => {
                let tok = self.bump().unwrap();
                Pattern::Literal(LiteralPattern {
                    value: self.lexer_slice(&tok),
                    span: tok.span,
                })
            }
            Some(TokenKind::LParen) => {
                let lo = self.bump().unwrap().span;
                if self.eat(TokenKind::RParen) {
                    return Pattern::Tuple(TuplePattern { elements: Vec::new(), rest: false, span: lo });
                }
                let mut elements = Vec::new();
                let mut rest = false;
                while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                    if self.eat(TokenKind::DotDot) {
                        rest = true;
                        break;
                    }
                    elements.push(self.parse_pattern());
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen);
                Pattern::Tuple(TuplePattern {
                    elements,
                    rest,
                    span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                })
            }
            Some(TokenKind::And) => {
                let lo = self.bump().unwrap().span;
                let is_mut = self.eat(TokenKind::KwMut);
                let inner = self.parse_pattern();
                Pattern::Ref(RefPattern {
                    is_mut,
                    inner: Box::new(inner),
                    span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                })
            }
            _ => Pattern::Wildcard(Span::DUMMY),
        }
    }

    pub fn parse_ty(&mut self) -> Ty {
        let ty = self.parse_ty_atom();

        while self.at(TokenKind::Plus) {
            panic!("trait object syntax not yet supported");
        }

        ty
    }

    fn parse_ty_atom(&mut self) -> Ty {
        match self.peek_tok().map(|t| t.kind.clone()) {
            Some(TokenKind::Ident) | Some(TokenKind::KwSelfType) => {
                let path = self.parse_path();
                let span = path.span;
                Ty::Path(PathTy { path, span })
            }
            Some(TokenKind::And) => {
                let lo = self.bump().unwrap().span;
                let is_mut = self.eat(TokenKind::KwMut);
                let inner = self.parse_ty();
                Ty::Ref(RefTy {
                    is_mut,
                    inner: Box::new(inner),
                    span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                })
            }
            Some(TokenKind::LParen) => {
                let lo = self.bump().unwrap().span;
                if self.eat(TokenKind::RParen) {
                    Ty::Unit(lo)
                } else {
                    let first = self.parse_ty();
                    if self.eat(TokenKind::Comma) {
                        let mut types = vec![first];
                        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                            types.push(self.parse_ty());
                            if !self.eat(TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(TokenKind::RParen);
                        Ty::Tuple(TupleTy { types, span: lo.to(self.peek_tok().map_or(lo, |t| t.span)) })
                    } else {
                        self.expect(TokenKind::RParen);
                        first
                    }
                }
            }
            Some(TokenKind::LBracket) => {
                let lo = self.bump().unwrap().span;
                let elem = self.parse_ty();
                if self.eat(TokenKind::Semi) {
                    let len = self.parse_expr();
                    self.expect(TokenKind::RBracket);
                    Ty::Array(ArrayTy {
                        elem: Box::new(elem),
                        len,
                        span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                    })
                } else {
                    self.expect(TokenKind::RBracket);
                    Ty::Slice(SliceTy {
                        elem: Box::new(elem),
                        span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                    })
                }
            }
            Some(TokenKind::KwFn) => {
                let lo = self.bump().unwrap().span;
                self.expect(TokenKind::LParen);
                let mut params = Vec::new();
                while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                    params.push(self.parse_ty());
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen);
                let ret = if self.eat(TokenKind::RArrow) {
                    Box::new(self.parse_ty())
                } else {
                    Box::new(Ty::Unit(Span::DUMMY))
                };
                Ty::Fn(FnTy {
                    params,
                    ret,
                    span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
                })
            }
            Some(TokenKind::Underscore) => {
                let tok = self.bump().unwrap();
                Ty::Infer(tok.span)
            }
            _ => Ty::Infer(Span::DUMMY),
        }
    }

    fn parse_ty_with_path(&mut self, path: PathExpr) -> Ty {
        let span = path.span;
        Ty::Path(PathTy { path, span })
    }

    fn parse_path(&mut self) -> PathExpr {
        let lo = self.peek_tok().unwrap().span;
        let mut segments = Vec::new();

        loop {
            let tok = self.expect(TokenKind::Ident).unwrap_or_else(|| {
                let dummy = Token::new(TokenKind::Ident, lo);
                dummy
            });
            segments.push(PathSegment {
                name: self.lexer_slice(&tok),
                args: None,
            });

            if self.eat(TokenKind::PathSep) {
                continue;
            }
            break;
        }

        PathExpr {
            segments,
            span: lo.to(self.peek_tok().map_or(lo, |t| t.span)),
        }
    }

    fn parse_generic_args(&mut self) -> Option<Vec<Ty>> {
        if self.eat(TokenKind::Lt) {
            self.parse_generic_args_inner()
        } else {
            None
        }
    }

    fn parse_generic_args_inner(&mut self) -> Option<Vec<Ty>> {
        let mut args = Vec::new();
        while !self.at(TokenKind::Gt) && !self.at(TokenKind::Eof) {
            args.push(self.parse_ty());
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::Gt);
        if args.is_empty() {
            None
        } else {
            Some(args)
        }
    }

    fn lexer_slice(&self, tok: &Token) -> String {
        self.file.src[tok.span.lo.to_usize()..tok.span.hi.to_usize()].to_string()
    }

    fn peek_type_start(&self, tok: &Token) -> Option<()> {
        match tok.kind {
            TokenKind::Ident | TokenKind::KwSelfType | TokenKind::And
            | TokenKind::LParen | TokenKind::LBracket | TokenKind::KwFn
            | TokenKind::Underscore => Some(()),
            _ => None,
        }
    }

    fn postfix_op(&self, tok: &Token) -> Option<(OpKind, u8, u8)> {
        match tok.kind {
            TokenKind::LParen => Some((OpKind::Postfix(Box::new(Self::parse_call)), PREC_CALL, PREC_CALL)),
            TokenKind::LBracket => Some((OpKind::Postfix(Box::new(Self::parse_index)), PREC_CALL, PREC_CALL)),
            TokenKind::Dot => Some((OpKind::Postfix(Box::new(Self::parse_field)), PREC_FIELD, PREC_FIELD)),
            TokenKind::Question => Some((OpKind::Postfix(Box::new(Self::parse_try)), PREC_CALL, PREC_CALL)),
            _ => None,
        }
    }

    fn parse_call_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        while !self.at(TokenKind::RParen) && self.peek_tok().is_some() {
            args.push(self.parse_expr());
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen);
        args
    }

    fn parse_call(parser: &mut Parser, func: Expr) -> Expr {
        let func_span = func.span();
        // LParen already consumed by Pratt loop's self.bump()
        let args = parser.parse_call_args();
        let span = func_span.to(parser.peek_tok().map_or(func_span, |t| t.span));
        Expr::Call(CallExpr { func: Box::new(func), args, span })
    }

    fn parse_index(parser: &mut Parser, base: Expr) -> Expr {
        let base_span = base.span();
        let index = parser.parse_expr();
        parser.expect(TokenKind::RBracket);
        let span = base_span.to(parser.peek_tok().map_or(base_span, |t| t.span));
        Expr::Index(IndexExpr { base: Box::new(base), index: Box::new(index), span })
    }

    fn parse_field(parser: &mut Parser, base: Expr) -> Expr {
        let base_span = base.span();
        if parser.at(TokenKind::Integer) {
            let tok = parser.bump().unwrap();
            let field = parser.lexer_slice(&tok);
            Expr::Field(FieldExpr { base: Box::new(base), field, span: base_span.to(tok.span) })
        } else {
            let tok = parser.expect(TokenKind::Ident).unwrap();
            let field = parser.lexer_slice(&tok);
            Expr::Field(FieldExpr { base: Box::new(base), field, span: base_span.to(tok.span) })
        }
    }

    fn parse_try(parser: &mut Parser, base: Expr) -> Expr {
        let span = base.span().to(parser.peek_tok().map_or(base.span(), |t| t.span));
        Expr::Try(TryExpr { expr: Box::new(base), span })
    }

    fn infix_op(&self, tok: &Token) -> Option<(OpKind, u8, u8)> {
        let (kind, rbp) = match tok.kind {
            TokenKind::OrOr => (OpKind::Infix(BinOp::OrOr), PREC_OR),
            TokenKind::AndAnd => (OpKind::Infix(BinOp::AndAnd), PREC_AND),
            TokenKind::EqEq => (OpKind::Infix(BinOp::Eq), PREC_CMP),
            TokenKind::Ne => (OpKind::Infix(BinOp::Ne), PREC_CMP),
            TokenKind::Lt => (OpKind::Infix(BinOp::Lt), PREC_CMP),
            TokenKind::Gt => (OpKind::Infix(BinOp::Gt), PREC_CMP),
            TokenKind::Le => (OpKind::Infix(BinOp::Le), PREC_CMP),
            TokenKind::Ge => (OpKind::Infix(BinOp::Ge), PREC_CMP),
            TokenKind::Or => (OpKind::Infix(BinOp::BitOr), PREC_BITOR),
            TokenKind::Caret => (OpKind::Infix(BinOp::BitXor), PREC_BITXOR),
            TokenKind::And => (OpKind::Infix(BinOp::BitAnd), PREC_BITAND),
            TokenKind::Shl => (OpKind::Infix(BinOp::Shl), PREC_SHIFT),
            TokenKind::Shr => (OpKind::Infix(BinOp::Shr), PREC_SHIFT),
            TokenKind::Plus => (OpKind::Infix(BinOp::Add), PREC_ADD),
            TokenKind::Minus => (OpKind::Infix(BinOp::Sub), PREC_ADD),
            TokenKind::Star => (OpKind::Infix(BinOp::Mul), PREC_MUL),
            TokenKind::Slash => (OpKind::Infix(BinOp::Div), PREC_MUL),
            TokenKind::Percent => (OpKind::Infix(BinOp::Rem), PREC_MUL),
            TokenKind::Eq => (OpKind::Assign, PREC_ASSIGN),
            TokenKind::PlusEq => (OpKind::Infix(BinOp::AddAssign), PREC_ASSIGN),
            TokenKind::MinusEq => (OpKind::Infix(BinOp::SubAssign), PREC_ASSIGN),
            TokenKind::StarEq => (OpKind::Infix(BinOp::MulAssign), PREC_ASSIGN),
            TokenKind::SlashEq => (OpKind::Infix(BinOp::DivAssign), PREC_ASSIGN),
            TokenKind::PercentEq => (OpKind::Infix(BinOp::RemAssign), PREC_ASSIGN),
            TokenKind::AndEq => (OpKind::Infix(BinOp::BitAndAssign), PREC_ASSIGN),
            TokenKind::OrEq => (OpKind::Infix(BinOp::BitOrAssign), PREC_ASSIGN),
            TokenKind::CaretEq => (OpKind::Infix(BinOp::BitXorAssign), PREC_ASSIGN),
            TokenKind::ShlEq => (OpKind::Infix(BinOp::ShlAssign), PREC_ASSIGN),
            TokenKind::ShrEq => (OpKind::Infix(BinOp::ShrAssign), PREC_ASSIGN),
            TokenKind::DotDot => return Some((OpKind::Range(false), PREC_RANGE_L, PREC_RANGE_R)),
            TokenKind::DotDotEq => return Some((OpKind::Range(true), PREC_RANGE_L, PREC_RANGE_R)),
            _ => return None,
        };
        Some((kind, rbp, rbp))
    }
}

enum OpKind {
    Infix(BinOp),
    Assign,
    Range(bool),
    Postfix(Box<dyn Fn(&mut Parser, Expr) -> Expr>),
}

const PREC_RANGE_L: u8 = 1;
const PREC_RANGE_R: u8 = 1;
const PREC_ASSIGN: u8 = 2;
const PREC_OR: u8 = 3;
const PREC_AND: u8 = 4;
const PREC_CMP: u8 = 5;
const PREC_BITOR: u8 = 6;
const PREC_BITXOR: u8 = 7;
const PREC_BITAND: u8 = 8;
const PREC_SHIFT: u8 = 9;
const PREC_ADD: u8 = 10;
const PREC_MUL: u8 = 11;
const PREC_PREFIX: u8 = 20;
const PREC_CALL: u8 = 22;
const PREC_FIELD: u8 = 22;

fn split_suffix(raw: &str) -> (String, Option<String>) {
    if let Some(pos) = raw.find(|c: char| c == 'i' || c == 'u' || c == 'f') {
        if pos > 0 && !raw.as_bytes()[pos - 1].is_ascii_digit() && raw.as_bytes()[pos - 1] != b'.' {
            return (raw.to_string(), None);
        }
        let suffix_start = pos;
        let suffix = &raw[suffix_start..];
        let value = &raw[..suffix_start];
        let valid_suffix = suffix.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if valid_suffix && !suffix.is_empty() {
            (value.to_string(), Some(suffix.to_string()))
        } else {
            (raw.to_string(), None)
        }
    } else {
        (raw.to_string(), None)
    }
}

impl<'a> Parser<'a> {
    fn parse_struct_literal_body(&mut self) -> Vec<(String, Expr)> {
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) && self.peek_tok().is_some() {
            if self.eat(TokenKind::DotDot) {
                break;
            }
            let field_name = self.expect(TokenKind::Ident)
                .map(|t| self.lexer_slice(&t)).unwrap_or_default();
            let value = if self.eat(TokenKind::Colon) {
                self.parse_expr()
            } else {
                Expr::Ident(IdentExpr { name: field_name.clone(), span: Span::DUMMY })
            };
            fields.push((field_name, value));
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        fields
    }
}
