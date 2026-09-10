use std::collections::HashMap;

use crate::frontend::{
    ast::{Expr, Literal, Program, Stmt},
    errors::ResultStmt,
    lookups::BindingPower,
    parser::Parser,
    tokens::TokenKind,
};

pub struct Preprocessor {
    pub aliases: HashMap<String, Box<Expr>>,
    pub ast: Option<Program>,
}

impl Preprocessor {
    pub fn new(ast: Option<Program>, aliases: Option<HashMap<String, Box<Expr>>>) -> Self {
        Self {
            aliases: if let Some(hs) = aliases {
                hs
            } else {
                HashMap::new()
            },
            ast,
        }
    }

    pub fn add_alias(&mut self, name: String, expr: Box<Expr>) {
        self.aliases.insert(name, expr);
    }

    pub fn resolve(&self, name: &str) -> Option<&Box<Expr>> {
        self.aliases.get(name)
    }

    pub fn resolve_stmt(&self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::Alias(name, expr) => {
                Stmt::Alias(name.clone(), self.resolve_expr(expr.clone()).into())
            }

            Stmt::Expr(expr) => Stmt::Expr(Box::new(self.resolve_expr(expr.clone()))),

            Stmt::For(condition, body) => Stmt::For(
                Box::new(self.resolve_expr(condition.clone())),
                Box::new(self.resolve_expr(body.clone())),
            ),

            Stmt::If {
                condition,
                then_branch,
                elifs,
                else_branch,
            } => Stmt::If {
                condition: Box::new(self.resolve_expr(condition.clone())),
                then_branch: Box::new(self.resolve_expr(then_branch.clone())),

                elifs: elifs
                    .iter()
                    .map(|(condition, body)| {
                        (
                            Box::new(self.resolve_expr(condition.clone())),
                            Box::new(self.resolve_expr(body.clone())),
                        )
                    })
                    .collect(),

                else_branch: else_branch
                    .as_ref()
                    .map(|body| Box::new(self.resolve_expr(body.clone()))),
            },

            Stmt::Var {
                name,
                expr,
                imut,
                dynm,
                typ,
            } => Stmt::Var {
                name: name.to_string(),
                expr: expr
                    .as_ref()
                    .map(|x| self.resolve_expr(Box::new(x.clone()))),
                imut: *imut,
                dynm: *dynm,
                typ: typ.clone(),
            },

            Stmt::Metadata(m, s) => Stmt::Metadata(*m, s.clone()),

            Stmt::While(c, body) => Stmt::While(
                Box::new(self.resolve_expr(c.clone())),
                Box::new(self.resolve_expr(body.clone())),
            ),

            Stmt::Function(name, args, ret, expr) => Stmt::Function(
                name.to_string(),
                args.to_vec(),
                ret.clone(),
                Box::new(self.resolve_expr(expr.clone())),
            ),
        }
    }

    pub fn resolve_expr(&self, expr: Box<Expr>) -> Expr {
        match *expr {
            Expr::Literal(Literal::Symbol(symbol)) => {
                if let Some(alias) = self.resolve(&symbol) {
                    self.resolve_expr(alias.clone())
                } else {
                    Expr::Literal(Literal::Symbol(symbol))
                }
            }

            Expr::Literal(l) => Expr::Literal(l),

            Expr::Binary(left, op, right) => Expr::Binary(
                Box::new(self.resolve_expr(left)),
                op,
                Box::new(self.resolve_expr(right)),
            ),

            Expr::Assign(left, op, right) => Expr::Assign(
                Box::new(self.resolve_expr(left)),
                op,
                Box::new(self.resolve_expr(right)),
            ),

            Expr::Block(stmts) => {
                Expr::Block(stmts.iter().map(|stmt| self.resolve_stmt(stmt)).collect())
            }

            Expr::Call(c, args) => Expr::Call(
                Box::new(self.resolve_expr(c)),
                args.iter()
                    .map(|a| self.resolve_expr(Box::new(a.clone())))
                    .collect(),
            ),

            Expr::Convert(b, t) => Expr::Convert(Box::new(self.resolve_expr(b)), t),

            Expr::Index(l, i) => Expr::Index(
                Box::new(self.resolve_expr(l)),
                Box::new(self.resolve_expr(i)),
            ),

            Expr::Map(m) => Expr::Map(
                m.iter()
                    .map(|(k, v)| {
                        (
                            self.resolve_expr(Box::new(k.clone())),
                            v.as_ref().map(|x| self.resolve_expr(Box::new(x.clone()))),
                        )
                    })
                    .collect(),
            ),

            Expr::Member(m, f) => Expr::Member(Box::new(self.resolve_expr(m)), f),

            Expr::Of(e, l) => Expr::Of(
                Box::new(self.resolve_expr(e)),
                Box::new(self.resolve_expr(l)),
            ),

            Expr::Postfix(op, e) => Expr::Postfix(op, Box::new(self.resolve_expr(e))),

            Expr::Prefix(op, e) => Expr::Prefix(op, Box::new(self.resolve_expr(e))),

            Expr::Range(a, b, c, d) => Expr::Range(
                Box::new(self.resolve_expr(a)),
                Box::new(self.resolve_expr(b)),
                Box::new(self.resolve_expr(c)),
                d,
            ),

            Expr::Vector(v) => Expr::Vector(
                v.iter()
                    .map(|e| self.resolve_expr(Box::new(e.clone())))
                    .collect(),
            ),

            Expr::Tuple(v) => Expr::Tuple(
                v.iter()
                    .map(|e| self.resolve_expr(Box::new(e.clone())))
                    .collect(),
            ),

            Expr::TypeOf(e) => Expr::TypeOf(Box::new(self.resolve_expr(e))),

            Expr::Type(t) => Expr::Type(t),
        }
    }

    pub fn resolve_ast(&mut self) {
        if let Some(program) = self.ast.take() {
            self.ast = Some(Program(
                program
                    .0
                    .iter()
                    .map(|stmt| self.resolve_stmt(stmt))
                    .collect(),
            ));
        }
    }
}

impl Parser {
    pub fn parse_alias(&mut self) -> ResultStmt {
        self.eat();

        let name = self.expect(TokenKind::IDENT)?.value.clone();

        self.expect(TokenKind::ASSIGN)?;

        let expr = self.parse_expr(BindingPower::DEFAULT)?;

        self.preproc.add_alias(name.clone(), expr.clone());

        self.expect(TokenKind::SC)?;

        Ok(Box::new(Stmt::Alias(name, expr)))
    }
}
