use crate::{misc::*, selector::*, FAUNA};
use faunadb::{error::Error as FaunaError, prelude::*};
use futures::future::Future;
use serde_json::{json, Value as JsonValue};

pub struct Post;

#[derive(Extract, Response, Debug, Clone)]
#[web(header(name = "content-type", value = "application/json"))]
pub struct PostData {
    title: String,
    age_limit: u16,
}

impl_web! {
    impl Post {
        #[get("/posts")]
        #[content_type("application/json")]
        fn index(&self) -> impl Future<Item = JsonValue, Error = FaunaError> + Send {
            let query = Selector::from_index("all_posts")
                .fields(vec!["title", "age_limit"])
                .into_query();

            FAUNA
                .query(query)
                .map_err(|e| dbg!(e))
                .map(|resp| {
                    let res = resp.resource;

                    json!({
                        "data": res["data"],
                        "before": res["before"],
                        "after": res["after"],
                    })
                })
        }

        #[post("/posts")]
        #[content_type("application/json")]
        fn create(&self, body: PostData) -> impl Future<Item = HttpCreated, Error = FaunaError> + Send {
            FAUNA
                .query(Create::new(Class::find("posts"), Object::from(body)))
                .map(|resp| {
                    let resource = resp.resource;
                    let reference = resource.get_reference().unwrap();

                    HttpCreated {
                        location: format!("http://localhost:8080/posts/{}", reference.id),
                    }
                })
        }

        #[put("/posts/:id")]
        #[content_type("application/json")]
        fn update(&self, id: String, body: PostData) -> impl Future<Item = HttpResponse, Error = FaunaError> + Send {
            let mut params = UpdateParams::new();
            params.data(Object::from(body));

            let mut reference = Ref::instance(id);
            reference.set_class("posts");

            FAUNA
                .query(Update::new(reference, params))
                .then(|result| {
                    Ok(HttpResponse::from(result))
                })
        }

        #[delete("/posts/:id")]
        #[content_type("application/json")]
        fn delete(&self, id: String) -> impl Future<Item = HttpResponse, Error = FaunaError> + Send {
            let mut reference = Ref::instance(id);
            reference.set_class("posts");

            FAUNA
                .query(Delete::new(reference))
                .then(|result| Ok(HttpResponse::from(result)))
        }

        #[get("/posts/:id")]
        #[content_type("application/json")]
        fn find(&self, id: String) -> impl Future<Item = HttpResponse, Error = FaunaError> + Send {
            let mut reference = Ref::instance(id);
            reference.set_class("posts");

            FAUNA
                .query(Get::instance(reference))
                .then(|result| Ok(HttpResponse::from(result)))
        }
    }
}

impl<'a> From<PostData> for Object<'a> {
    fn from(post: PostData) -> Self {
        let mut obj = Object::default();
        obj.insert("title", post.title);
        obj.insert("age_limit", post.age_limit);

        obj
    }
}
