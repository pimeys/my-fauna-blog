#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

use faunadb::prelude::*;
use futures::{
    future,
    stream::Stream,
    sync::oneshot::{channel, Receiver},
    Future,
};
use hyper::{header, service::service_fn, Body, Method, Request, Response, Server};
use lazy_static::lazy_static;
use std::{env, net::ToSocketAddrs, thread};
use tokio::runtime::Runtime;
use tokio_signal::unix::{Signal, SIGINT};

lazy_static! {
    pub static ref FAUNA: Client = {
        let secret = env::var("SECRET").unwrap_or_else(|_| String::from("secret"));
        let mut builder = ClientBuilder::new(secret.as_str());

        if let Ok(uri) = env::var("FAUNA_URI") {
            builder.uri(uri);
        }

        builder.build().unwrap()
    };
}

type ResponseFuture = Box<Future<Item = Response<Body>, Error = Error> + Send + 'static>;

#[derive(Debug)]
enum Error {
    Other,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ERROR HAPPENED")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "ERROR ERROR"
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Post {
    title: String,
    tags: Vec<String>,
}

impl<'a> From<Post> for Object<'a> {
    fn from(post: Post) -> Self {
        let mut obj = Object::default();
        obj.insert("title", post.title);
        obj.insert("tags", Array::from(post.tags));

        obj
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

    fn service<'a>(req: Request<Body>) -> ResponseFuture {
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/posts") => {
                let (_, body) = req.into_parts();

                let future = body.concat2().and_then(move |body| {
                    let post: Post = serde_json::from_slice(&body).unwrap();

                    FAUNA
                        .query(Create::new(Class::find("posts"), Object::from(post)))
                        .then(|resp| match resp {
                            Ok(result) => {
                                let resource = result.resource.as_object().unwrap();
                                let reference = resource
                                    .get("ref")
                                    .and_then(|res| res.as_reference())
                                    .unwrap();

                                let location =
                                    format!("http://localhost:4200/posts/{}", reference.id);

                                let mut builder = Response::builder();
                                builder.header(header::LOCATION, location);
                                builder.status(201);

                                Ok(builder.body(Body::empty()).unwrap())
                            }
                            Err(e) => {
                                error!("Error with Fauna: {:?}", e);

                                let mut builder = Response::builder();
                                builder.status(500);

                                Ok(builder.body(Body::from("INTERNAL SERVER ERROR")).unwrap())
                            }
                        })
                });

                Box::new(future.map_err(|_| Error::Other))
            }
            _ => {
                let body = Body::from("Hello, world!");
                let res = Response::builder().status(200).body(body);

                Box::new(future::ok(res.unwrap()))
            }
        }
    }

    fn delete_database(&mut self) {
        self.run_expr(Delete::new(Index::find("posts_by_title")));
        self.run_expr(Delete::new(Index::find("posts_by_tags_with_title")));
        self.delete_class("posts");
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

    fn create_schema(&mut self) {
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
}

fn main() {
    pretty_env_logger::init();

    let mut blog = Blog::new();

    match env::var("MIGRATE") {
        Ok(ref cmd) if cmd == "delete" => blog.delete_database(),
        Ok(_) => blog.create_schema(),
        _ => {
            let (tx, rx) = channel();

            let server = thread::spawn(move || {
                info!("Server listening on port 4200");
                blog.run(format!("0.0.0.0:4200"), rx);
                info!("Server going down...");
            });

            let _ = Signal::new(SIGINT)
                .flatten_stream()
                .into_future()
                .and_then(|_| {
                    if let Err(error) = tx.send(()) {
                        error!("Error sending the shutdown signal {:?}", error);
                    };

                    server.join().unwrap();
                    Ok(())
                })
                .wait();
        }
    }
}
