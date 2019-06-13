#[macro_use]
extern crate tower_web;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

// mod blog; let's rethink this
mod error;
mod post;

//use blog::Blog;
use faunadb::client::{Client, ClientBuilder};
use lazy_static::lazy_static;
use post::Post;
use std::env;
use tower_web::ServiceBuilder;

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

fn main() {
    pretty_env_logger::init();

    let addr = "127.0.0.1:8080".parse().expect("Invalid address");
    info!("Listening on http://{}", addr);

    ServiceBuilder::new().resource(Post).run(&addr).unwrap()
}
