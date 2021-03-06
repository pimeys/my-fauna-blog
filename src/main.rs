#![recursion_limit = "128"]

#[macro_use]
extern crate tower_web;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

mod migrate;
mod post;
mod tag;

pub(crate) mod misc;
pub(crate) mod selector;

use faunadb::client::Client;
use lazy_static::lazy_static;
use migrate::Migrate;
use post::Post;
use std::env;
use tag::Tag;
use tower_web::ServiceBuilder;

lazy_static! {
    pub static ref FAUNA: Client = {
        let secret = env::var("SECRET").unwrap_or_else(|_| String::from("secret"));
        let mut builder = Client::builder(secret.as_str());

        if let Ok(uri) = env::var("FAUNA_URI") {
            builder.uri(uri);
        }

        builder.build().unwrap()
    };
}

fn main() {
    pretty_env_logger::init();

    match env::var("MIGRATE") {
        Ok(cmd) => {
            let mut migrate = Migrate::new();

            if cmd == "delete" {
                migrate.delete_database();
            } else {
                migrate.create_schema();
            }
        }
        _ => {
            let addr = "127.0.0.1:8080".parse().expect("Invalid address");
            info!("Listening on http://{}", addr);

            ServiceBuilder::new()
                .resource(Post)
                .resource(Tag)
                .run(&addr)
                .unwrap()
        }
    }
}
