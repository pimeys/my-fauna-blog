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
        self.run_expr(Delete::new(Index::find("posts_by_tags_with_title")));
        self.delete_class("posts");
    }

    pub fn create_schema(&mut self) {
        self.create_class("posts");

        {
            let mut params = IndexParams::new("posts_by_title", Class::find("posts"));
            params.terms(vec![Term::field(vec!["data", "title"])]);
            self.run_expr(CreateIndex::new(params));

            let mut params = IndexParams::new("posts_by_tags_with_title", Class::find("posts"));
            params.terms(vec![Term::field(vec!["data", "tags"])]);
            params.values(vec![IndexValue::field(vec!["data", "title"])]);
            self.run_expr(CreateIndex::new(params));
        }
    }

    fn create_class(&mut self, name: &str) {
        trace!("Create class {}.", name);
        self.run_expr(CreateClass::new(ClassParams::new(name)));

        self.run_expr(CreateIndex::new(IndexParams::new(
            format!("all_{}", name),
            Class::find(name),
        )));
    }

    fn delete_class(&mut self, name: &str) {
        trace!("Delete class {}.", name);
        self.run_expr(Delete::new(Index::find(format!("all_{}", name))));
        self.run_expr(Delete::new(Index::find(format!("schema_{}", name))));
        self.run_expr(Delete::new(Class::find(name)));
    }

    fn run_expr<'a>(&mut self, expr: impl Into<Expr<'a>>) {
        self.runtime.block_on(FAUNA.query(expr)).unwrap();
    }
}
