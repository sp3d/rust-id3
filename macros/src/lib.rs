#![crate_name = "id3_macros"]
#![crate_type = "dylib"]

#![feature(plugin_registrar, quote, default_type_params, macro_rules, globs)]

extern crate syntax;
extern crate rustc;

use syntax::ast::{self, TokenTree, LitStr, LitChar, ExprLit};
use syntax::codemap::{Span};
use syntax::ext::base::*;
use rustc::plugin::Registry;
use syntax::ext::build::AstBuilder;

#[plugin_registrar]
#[doc(hidden)]
pub fn macro_registrar(reg: &mut Registry) {
    reg.register_macro("b", expand_b);
}

/// derived from bytes! macro, to replace *bytes!("foo")
pub fn expand_b<'cx>(cx: &'cx mut ExtCtxt,
                              sp: Span,
                              tts: &[ast::TokenTree])
                              -> Box<MacResult+'cx> {
    // Gather all argument expressions
    let exprs = match get_exprs_from_tts(cx, sp, tts) {
        None => return DummyResult::expr(sp),
        Some(e) => e,
    };
    let mut bytes = Vec::new();
    let mut err = false;

    for expr in exprs.iter() {
        match expr.node {
            // expression is a literal
            ast::ExprLit(ref lit) => match lit.node {
                // string literal, push each byte to vector expression
                ast::LitStr(ref s, _) => {
                    for byte in s.get().bytes() {
                        bytes.push(cx.expr_u8(expr.span, byte));
                    }
                }

                // u8 literal, push to vector expression
                ast::LitInt(v, ast::UnsignedIntLit(ast::TyU8)) => {
                    if v > 0xFF {
                        cx.span_err(expr.span, "too large u8 literal in bytes!");
                        err = true;
                    } else {
                        bytes.push(cx.expr_u8(expr.span, v as u8));
                    }
                }

                // integer literal, push to vector expression
                ast::LitInt(_, ast::UnsuffixedIntLit(ast::Minus)) => {
                    cx.span_err(expr.span, "negative integer literal in bytes!");
                    err = true;
                }
                ast::LitInt(v, ast::UnsuffixedIntLit(ast::Plus)) => {
                    if v > 0xFF {
                        cx.span_err(expr.span, "too large integer literal in bytes!");
                        err = true;
                    } else {
                        bytes.push(cx.expr_u8(expr.span, v as u8));
                    }
                }

                // char literal, push to vector expression
                ast::LitChar(v) => {
                    if v.is_ascii() {
                        bytes.push(cx.expr_u8(expr.span, v as u8));
                    } else {
                        cx.span_err(expr.span, "non-ascii char literal in bytes!");
                        err = true;
                    }
                }

                _ => {
                    cx.span_err(expr.span, "unsupported literal in bytes!");
                    err = true;
                }
            },

            _ => {
                cx.span_err(expr.span, "non-literal in bytes!");
                err = true;
            }
        }
    }

    // For some reason using quote_expr!() here aborts if we threw an error.
    // I'm assuming that the end of the recursive parse tricks the compiler
    // into thinking this is a good time to stop. But we'd rather keep going.
    if err {
        // Since the compiler will stop after the macro expansion phase anyway, we
        // don't need type info, so we can just return a DummyResult
        return DummyResult::expr(sp);
    }

    let e = cx.expr_vec(sp, bytes);
    /*let len = bytes.len();
    let ty = cx.ty(sp, ast::TyFixedLengthVec(cx.ty_ident(sp, cx.ident_of("u8")),
                                             cx.expr_uint(sp, len)));
    let item = cx.item_static(sp, cx.ident_of("BYTES"), ty, ast::MutImmutable, e);
    let ret = cx.expr_ident(sp, cx.ident_of("BYTES"));
    let ret = cx.expr_addr_of(sp, ret);
    let e = cx.expr_block(cx.block(sp, vec![cx.stmt_item(sp, item)],
                                   Some(ret)));*/
    MacExpr::new(e)
}
