use crate::lir::{Expr, Value};
use crate::name::Name;
use crate::util::Map;

#[derive(Debug, Clone)]
struct Ctxt {
    vars: Map<Name, Value>,
}

impl Ctxt {
    fn empty() -> Self {
        Self { vars: Map::new() }
    }
}

pub fn eval_root_expr(expr: Expr) -> Value {
    eval_expr(Ctxt::empty(), expr)
}

fn eval_expr(mut cx: Ctxt, expr: Expr) -> Value {
    match expr {
        Expr::Var(var) => cx.vars[&var.name].clone(),

        Expr::U64(n) => Value::U64(n),
        Expr::Record(fields) => {
            Value::Record(fields.into_iter().map(|(n, e)| (n, eval_expr(cx.clone(), e))).collect())
        }
        Expr::UntaggedUnion { ty: _, field: _, value } => eval_expr(cx, *value),

        Expr::Box(val) => Value::Box(Box::new(eval_expr(cx, *val))),
        Expr::Deref(ptr) => {
            let ptr = eval_expr(cx, *ptr);
            match ptr {
                Value::Box(val) => *val,
                _ => panic!(),
            }
        }

        Expr::Select { record, field } => {
            let record = eval_expr(cx, *record);
            match record {
                Value::Record(fields) => fields[&field].clone(),
                _ => panic!(),
            }
        }

        Expr::Switch { subj, cases, default } => {
            let subj = cx.vars[&subj.name].clone();
            match subj {
                Value::U64(subj_val) => {
                    let case_body = cases
                        .get(&subj_val)
                        .or(default.as_deref())
                        .expect("no matching case found");
                    eval_expr(cx, case_body.clone())
                }
                _ => panic!(),
            }
        }
        Expr::Let { binder, value, body } => {
            let value = eval_expr(cx.clone(), *value);
            cx.vars.insert(binder.name, value);
            eval_expr(cx, *body)
        }
    }
}
