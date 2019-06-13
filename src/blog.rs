use super::{error::Error, post::Post, ResponseFuture, FAUNA};
use faunadb::prelude::*;
use futures::{future, stream::Stream, sync::oneshot::Receiver, Future};
use hyper::{service::service_fn, Body, Method, Request, Response, Server};
use std::net::ToSocketAddrs;
use tokio::runtime::Runtime;

pub struct Service;

impl_web! {
    impl Service {
        #[post("/posts")]
        fn create_post(&self, )
    }
}

pub struct Blog {
    runtime: Runtime,
}

impl Blog {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
        }
    }

    pub fn run(&mut self, address: String, kill_switch: Receiver<()>) {
        let mut addr_iter = address.to_socket_addrs().unwrap();
        let addr = addr_iter.next().unwrap();

        let server = Server::bind(&addr)
            .serve(move || service_fn(move |req: Request<Body>| Self::service(req))); // No table service :ADAsgsrehawj

        let _ = self.runtime.block_on(server.select2(kill_switch));
    }

    pub fn service<'a>(req: Request<Body>) -> ResponseFuture {
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/posts") => {
                let (_, body) = req.into_parts();

                let future = body
                    .concat2()
                    .map_err(|_| Error::Other)
                    .and_then(move |body| {
                        let post: Post = serde_json::from_slice(&body).unwrap();
                        post.create()
                    });

                Box::new(future.map_err(|_| Error::Other))
            }
            (&Method::GET, "")
            _ => {
                let body = Body::from("Hello, world!");
                let res = Response::builder().status(200).body(body);

                Box::new(future::ok(res.unwrap()))
            }
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

    fn run_expr<'a>(&mut self, expr: impl Into<Expr<'a>>) {
        self.runtime.block_on(FAUNA.query(expr)).unwrap();
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
}
