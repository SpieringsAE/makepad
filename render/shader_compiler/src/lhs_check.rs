
use crate::env::Env;
use makepad_live_parser::LiveError;
use makepad_live_parser::LiveErrorOrigin;
use makepad_live_parser::live_error_origin;
use makepad_live_parser::Span;
use crate::shaderast::Ident;
use crate::shaderast::ExprKind;
use crate::shaderast::BinOp;
use crate::shaderast::UnOp;
use crate::shaderast::Expr;
use crate::shaderast::IdentPath;
use crate::shaderast::Lit;
use crate::shaderast::VarKind;
use std::cell::Cell;

pub struct LhsChecker<'a, 'b> {
    pub env: &'a Env<'b>,
}

impl<'a, 'b> LhsChecker<'a, 'b> {
    pub fn lhs_check_expr(&mut self, expr: &Expr) -> Result<(), LiveError> {
        match expr.kind {
            ExprKind::Cond {
                span,
                ref expr,
                ref expr_if_true,
                ref expr_if_false,
                ..
            } => self.lhs_check_cond_expr(span, expr, expr_if_true, expr_if_false),
            ExprKind::Bin {
                span,
                op,
                ref left_expr,
                ref right_expr,
                ..
            } => self.lhs_check_bin_expr(span, op, left_expr, right_expr),
            ExprKind::Un {span, op, ref expr} => self.lhs_check_un_expr(span, op, expr),
            ExprKind::Field {
                span,
                ref expr,
                field_ident,
            } => self.lhs_check_field_expr(span, expr, field_ident),
            ExprKind::Index {
                span,
                ref expr,
                ref index_expr,
            } => self.lhs_check_index_expr(span, expr, index_expr),
            ExprKind::MethodCall {
                span,
                ..
            } => self.lhs_check_all_call_expr(span),
            ExprKind::PlainCall {
                span,
                ..
            } => self.lhs_check_all_call_expr(span),
            ExprKind::BuiltinCall {
                span,
                ..
            } => self.lhs_check_all_call_expr(span),
            ExprKind::ConsCall {
                span,
                ..
            } => self.lhs_check_all_call_expr(span),
            ExprKind::Var {
                span,
                ref kind,
                ident_path,
            } => self.lhs_check_var_expr(span, kind, ident_path),
            ExprKind::Lit {span, lit} => self.lhs_check_lit_expr(span, lit),
        }
    }
    
    fn lhs_check_cond_expr(
        &mut self,
        span: Span,
        _expr: &Expr,
        _expr_if_true: &Expr,
        _expr_if_false: &Expr,
    ) -> Result<(), LiveError> {
        return Err(LiveError {
            origin: live_error_origin!(),
            span,
            message: String::from("expression is not a valid left hand side"),
        });
    }
    
    fn lhs_check_bin_expr(
        &mut self,
        span: Span,
        _op: BinOp,
        _left_expr: &Expr,
        _right_expr: &Expr,
    ) -> Result<(), LiveError> {
        return Err(LiveError {
            origin:live_error_origin!(),
            span,
            message: String::from("expression is not a valid left hand side"),
        });
    }
    
    fn lhs_check_un_expr(&mut self, span: Span, _op: UnOp, _expr: &Expr) -> Result<(), LiveError> {
        return Err(LiveError {
            origin:live_error_origin!(),
            span,
            message: String::from("expression is not a valid left hand side"),
        });
    }
    
    fn lhs_check_all_call_expr(
        &mut self,
        span: Span,
    ) -> Result<(), LiveError> {
        return Err(LiveError {
            origin:live_error_origin!(),
            span,
            message: String::from("expression is not a valid left hand side"),
        });
    }
    
    fn lhs_check_field_expr(
        &mut self,
        _span: Span,
        expr: &Expr,
        _field_ident: Ident,
    ) -> Result<(), LiveError> {
        self.lhs_check_expr(expr)
    }
    
    fn lhs_check_index_expr(
        &mut self,
        _span: Span,
        expr: &Expr,
        _index_expr: &Expr,
    ) -> Result<(), LiveError> {
        self.lhs_check_expr(expr)
    }
    
    fn lhs_check_call_expr(
        &mut self,
        span: Span,
        _ident_path: IdentPath,
        _arg_exprs: &[Expr],
    ) -> Result<(), LiveError> {
        return Err(LiveError {
            origin:live_error_origin!(),
            span,
            message: String::from("expression is not a valid left hand side"),
        });
    }
    
    fn lhs_check_var_expr(
        &mut self,
        span: Span,
        kind: &Cell<Option<VarKind >>,
        _ident_path: IdentPath,
    ) -> Result<(), LiveError> {
        if let VarKind::MutLocal = kind.get().unwrap(){
            Ok(())
        }
        else{
            Err(LiveError {
                origin:live_error_origin!(),
                span,
                message: String::from("expression is not a valid left hand side"),
            })
        }
    }
    /*
    fn lhs_check_live_id_expr(
        &mut self,
        span: Span,
        _kind: &Cell<Option<VarKind>>,
        _id:LiveItemId,
        _ident: Ident,
    ) -> Result<(), LiveError> {
        return Err(LiveError {
            span,
            message: String::from("liveid is not a valid left hand side"),
        });
    }*/
    
    fn lhs_check_lit_expr(&mut self, _span: Span, _lit: Lit) -> Result<(), LiveError> {
        Ok(())
    }
}
