use super::FAUNA;
use faunadb::prelude::*;
use tokio::runtime::Runtime;

pub struct Migrate {
    runtime: Runtime,
}

impl Migrate {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
        }
    }

    pub fn delete_database(&mut self) {
        self.run_expr(Delete::new(Index::find("posts_by_title")));
        self.delete_class("posts");
    }

    pub fn create_schema(&mut self) {
        self.run_expr(CreateClass::new(ClassParams::new("posts")));

        {
            let mut params = IndexParams::new("posts_by_title", Class::find("posts"));
            params.terms(vec![Term::field(vec!["data", "title"])]);
            params.values(vec![
                IndexValue::field(vec!["ref"]),
                IndexValue::field(vec!["data", "title"]),
                IndexValue::field(vec!["data", "age_limit"]),
            ]);
            self.run_expr(CreateIndex::new(params));

            let mut params = IndexParams::new("all_posts", Class::find("posts"));
            params.values(vec![
                IndexValue::field(vec!["ref"]),
                IndexValue::field(vec!["data", "title"]),
                IndexValue::field(vec!["data", "age_limit"]),
            ]);
            self.run_expr(CreateIndex::new(params));
        }
    }

    fn delete_class(&mut self, name: &str) {
        trace!("Delete class {}.", name);
        self.run_expr(Delete::new(Index::find(format!("all_{}", name))));
        self.run_expr(Delete::new(Class::find(name)));
    }

    fn run_expr<'a>(&mut self, expr: impl Into<Expr<'a>>) {
        self.runtime.block_on(FAUNA.query(expr)).unwrap();
    }
}
