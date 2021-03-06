// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use syntax::ast::{self, TokenTree};
use syntax::codemap::Span;
use syntax::ext::base::*;
use syntax::ext::base;
use syntax::feature_gate;
use syntax::parse::token;
use syntax::parse::token::str_to_ident;
use syntax::ptr::P;

pub fn expand_syntax_ext<'cx>(cx: &'cx mut ExtCtxt, sp: Span, tts: &[TokenTree])
                              -> Box<base::MacResult+'cx> {
    if !cx.ecfg.enable_concat_idents() {
        feature_gate::emit_feature_err(&cx.parse_sess.span_diagnostic,
                                       "concat_idents",
                                       sp,
                                       feature_gate::GateIssue::Language,
                                       feature_gate::EXPLAIN_CONCAT_IDENTS);
        return base::DummyResult::expr(sp);
    }

    let mut res_str = String::new();
    for (i, e) in tts.iter().enumerate() {
        if i & 1 == 1 {
            match *e {
                TokenTree::Token(_, token::Comma) => {},
                _ => {
                    cx.span_err(sp, "concat_idents! expecting comma.");
                    return DummyResult::expr(sp);
                },
            }
        } else {
            match *e {
                TokenTree::Token(_, token::Ident(ident)) => {
                    res_str.push_str(&ident.name.as_str())
                },
                _ => {
                    cx.span_err(sp, "concat_idents! requires ident args.");
                    return DummyResult::expr(sp);
                },
            }
        }
    }
    let res = str_to_ident(&res_str);

    struct Result { ident: ast::Ident, span: Span };

    impl Result {
        fn path(&self) -> ast::Path {
            let segment = ast::PathSegment {
                identifier: self.ident,
                parameters: ast::PathParameters::none()
            };
            ast::Path { span: self.span, global: false, segments: vec![segment] }
        }
    }

    impl base::MacResult for Result {
        fn make_expr(self: Box<Self>) -> Option<P<ast::Expr>> {
            Some(P(ast::Expr {
                id: ast::DUMMY_NODE_ID,
                node: ast::ExprKind::Path(None, self.path()),
                span: self.span,
                attrs: None,
            }))
        }

        fn make_ty(self: Box<Self>) -> Option<P<ast::Ty>> {
            Some(P(ast::Ty {
                id: ast::DUMMY_NODE_ID,
                node: ast::TyKind::Path(None, self.path()),
                span: self.span,
            }))
        }
    }

    Box::new(Result { ident: res, span: sp })
}
