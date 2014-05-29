#![feature(managed_boxes, globs, macro_registrar, macro_rules, phase)]
#![crate_type = "dylib"]
#![crate_id = "ary"]

// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation of the ary![] macro
//! Thanks to kballard @ https://github.com/kballard
extern crate syntax;

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::*;
use syntax::ext::build::AstBuilder;
use syntax::parse;
use syntax::parse::token;
use syntax::ast::{Name, TokenTree};

fn expander(f: MacroExpanderFn) -> SyntaxExtension {
  NormalTT(box BasicMacroExpander {
    expander: f,
    span: None,
  }, None)
}

#[macro_registrar]
pub fn macro_registrar(register: |Name, SyntaxExtension|) {
  register(token::intern("ary"), expander(ary));
}

pub fn ary(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult> {
  let mut p = parse::new_parser_from_tts(cx.parse_sess(),
    cx.cfg(), Vec::from_slice(tts));

  let val_expr = p.parse_expr();
  p.expect(&token::COMMA);
  p.expect(&token::DOTDOT);
  
  // negative literals should not be a fatal error
  if p.eat(&token::BINOP(token::MINUS)) {
    p.span_err(p.last_span, "expected positive integral literal");
    return DummyResult::expr(sp);
  }

  let count_expr = cx.expand_expr(p.parse_expr());
  let count = match count_expr.node {
    ast::ExprLit(lit) => {
      match lit.node {
        ast::LitInt(i, _) | ast::LitIntUnsuffixed(i) if i > 0 => i as u64,
        ast::LitUint(u, _) if u > 0 => u,
        _ => {
          p.span_err(lit.span, "expected positive integral literal");
          return DummyResult::expr(sp);
        }
      }
    }
    _ => {
      p.span_err(count_expr.span, "expected integral literal");
      return DummyResult::expr(sp);
    }
  };

  let count = match count.to_uint() {
    None => {
      p.span_err(count_expr.span, "integral literal out of range");
      return DummyResult::expr(sp);
    }
    Some(x) => x
  };

  let exprs = Vec::from_fn(count, |_| val_expr.clone());
  MacExpr::new(cx.expr_vec(sp, exprs))
}
