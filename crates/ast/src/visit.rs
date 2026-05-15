use crate::{
    expr::Expr,
    item::Item,
    node::Stmt,
    pattern::Pattern,
    ty::Ty,
};

pub trait Visitor {
    fn visit_item(&mut self, _item: &Item) {}
    fn visit_expr(&mut self, _expr: &Expr) {}
    fn visit_stmt(&mut self, _stmt: &Stmt) {}
    fn visit_pattern(&mut self, _pattern: &Pattern) {}
    fn visit_ty(&mut self, _ty: &Ty) {}
}

pub trait MutVisitor {
    fn visit_item(&mut self, _item: &mut Item) {}
    fn visit_expr(&mut self, _expr: &mut Expr) {}
    fn visit_stmt(&mut self, _stmt: &mut Stmt) {}
    fn visit_pattern(&mut self, _pattern: &mut Pattern) {}
    fn visit_ty(&mut self, _ty: &mut Ty) {}
}
