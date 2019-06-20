use crate::{error::Error, FAUNA};
use faunadb::{error::Error as FaunaError, prelude::*};
use futures::future::Future;
use serde_json::{json, Value as JsonValue};
use std::convert::TryFrom;

pub struct Post;

#[derive(Extract, Response, Debug, Clone)]
#[web(header(name = "content-type", value = "application/json"))]
pub struct PostData {
    title: String,
    tags: Vec<String>,
}

#[derive(Debug, Response)]
#[web(status = "201")]
pub struct PostCreated {
    #[web(header)]
    location: String,
}

impl_web! {
    impl Post {
        #[post("/posts")]
        #[content_type("application/json")]
        fn create(&self, body: PostData) -> impl Future<Item = PostCreated, Error = FaunaError> + Send {
            FAUNA
                .query(Create::new(Class::find("posts"), Object::from(body)))
                .map(|resp| {
                    let resource = resp.resource;
                    let reference = resource.get_reference().unwrap();

                    PostCreated {
                        location: format!("http://localhost:8080/posts/{}", reference.id),
                    }
                })
        }

        #[get("/posts/:id")]
        #[content_type("application/json")]
        fn find(&self, id: String) -> impl Future<Item = JsonValue, Error = FaunaError> + Send {
            let mut reference = Ref::instance(id);
            reference.set_class("posts");

            FAUNA
                .query(Get::instance(reference))
                .map_err(|e| {
                    dbg!(&e);
                    e
                })
                .map(|resp| {
                    let res = resp.resource;

                    json!({
                        "id": res.get_reference().unwrap().id,
                        "title": res["data"]["title"],
                        "tags": res["data"]["tags"],
                    })
                })
        }
    }
}

impl<'a> From<PostData> for Object<'a> {
    fn from(post: PostData) -> Self {
        let mut obj = Object::default();
        obj.insert("title", post.title);
        obj.insert("tags", Array::from(post.tags));

        obj
    }
}

impl TryFrom<Value> for PostData {
    type Error = Error;

    fn try_from(value: Value) -> Result<PostData, Error> {
        fn lift<T>(opt: Option<T>) -> Result<T, Error> {
            opt.ok_or(Error::Conversion)
        }

        let mut obj = lift(value.into_object())?;
        let mut data = lift(lift(obj.remove("data"))?.into_object())?;

        let tag_values = lift(data.remove("tags"))?.into_array();

        let tags: Result<Vec<String>, Error> = lift(tag_values)?
            .into_iter()
            .map(|val| lift(val.into_string()))
            .collect();

        let tags = tags?;
        let title = lift(lift(data.remove("title"))?.into_string())?;

        Ok(PostData { title, tags })
    }
}
