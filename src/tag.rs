use crate::{misc::*, FAUNA};
use faunadb::{error::Error as FaunaError, prelude::*};
use futures::Future;
use serde_json::{json, Value as JsonValue};

pub struct Tag;

#[derive(Extract, Response, Debug, Clone)]
#[web(header(name = "content-type", value = "application/json"))]
pub struct TagData {
    name: String,
}

impl_web! {
    impl Tag {
        #[get("/posts/:post_id/tags")]
        #[content_type("application/json")]
        fn index(&self, post_id: String) -> impl Future<Item = JsonValue, Error = FaunaError> {
            let mut reference = Ref::instance(post_id.as_str());
            reference.set_class("posts");

            FAUNA
                .query(Get::instance(reference))
                .and_then(move |_| {
                    FAUNA
                        .query(Paginate::new(Match::new(Index::find("tags_by_post_id")).with_terms(post_id.as_str())))
                        .map_err(|e| dbg!(e))
                        .map(|resp| {
                            let res = resp.resource;

                            let data: Vec<JsonValue> = res["data"].as_array().unwrap().iter().map(|tag| {
                                json!({"id": tag[0], "name": tag[1]})
                            }).collect();

                            json!({
                                "data": data,
                                "before": res["before"],
                                "after": res["after"],
                            })
                        })
                })
        }

        #[post("/posts/:post_id/tags")]
        #[content_type("application/json")]
        fn create(&self, post_id: String, body: TagData) -> impl Future<Item = HttpCreated, Error = FaunaError> + Send {
            let mut reference = Ref::instance(post_id.as_str());
            reference.set_class("posts");

            FAUNA
                .query(Get::instance(reference))
                .map_err(|e| dbg!(e))
                .and_then(move |_| {
                    let mut obj = Object::default();
                    obj.insert("post_id", post_id.as_str());
                    obj.insert("name", body.name);

                    FAUNA
                        .query(Create::new(Class::find("tags"), obj))
                        .map(move |resp| {
                            let resource = resp.resource;
                            let reference = resource.get_reference().unwrap();

                            HttpCreated {
                                location: format!("http://localhost:8080/posts/{}/tags/{}", post_id, reference.id),
                            }
                        })
                })
        }
    }
}
