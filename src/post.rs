use crate::FAUNA;
use faunadb::{error::Error as FaunaError, prelude::*, FaunaResult};
use futures::future::Future;
use serde_json::{json, Value as JsonValue};

pub struct Post;

#[derive(Extract, Response, Debug, Clone)]
#[web(header(name = "content-type", value = "application/json"))]
pub struct PostData {
    title: String,
    age_limit: u16,
}

#[derive(Debug, Response)]
#[web(status = "201")]
pub struct PostCreated {
    #[web(header)]
    location: String,
}

#[derive(Debug, Response)]
#[web(either)]
pub enum PostResponse {
    Data(JsonValue),
    Error(PostError),
}

#[derive(Debug, Response)]
pub struct PostError {
    #[web(status)]
    status: u16,
}

impl From<FaunaResult<Response>> for PostResponse {
    fn from(result: FaunaResult<Response>) -> Self {
        match result {
            Ok(resp) => {
                let res = resp.resource;

                let payload = json!({
                    "id": res.get_reference().unwrap().id,
                    "title": res["data"]["title"],
                    "age_limit": res["data"]["age_limit"],
                });

                PostResponse::Data(payload)
            }
            Err(FaunaError::NotFound(_)) => PostResponse::Error(PostError { status: 404 }),
            Err(e) => {
                error!("FATAL ERROR: {:?}", e);
                PostResponse::Error(PostError { status: 500 })
            }
        }
    }
}

impl_web! {
    impl Post {
        #[get("/posts")]
        #[content_type("application/json")]
        fn index(&self) -> impl Future<Item = JsonValue, Error = FaunaError> + Send {
            FAUNA
                .query(Paginate::new(Match::new(Index::find("all_posts"))))
                .map_err(|e| dbg!(e))
                .map(|resp| {
                    let res = resp.resource;

                    let data: Vec<JsonValue> = res["data"].as_array().unwrap().iter().map(|blog| {
                        json!({"id": blog[0].as_reference().unwrap().id, "title": blog[1], "age_limit": blog[2]})
                    }).collect();

                    json!({
                        "data": data,
                        "before": res["before"],
                        "after": res["after"],
                    })
                })
        }

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

        #[put("/posts/:id")]
        #[content_type("application/json")]
        fn update(&self, id: String, body: PostData) -> impl Future<Item = PostResponse, Error = FaunaError> + Send {
            let mut params = UpdateParams::new();
            params.data(Object::from(body));

            let mut reference = Ref::instance(id);
            reference.set_class("posts");

            FAUNA
                .query(Update::new(reference, params))
                .then(|result| {
                    Ok(PostResponse::from(result))
                })
        }

        #[delete("/posts/:id")]
        #[content_type("application/json")]
        fn delete(&self, id: String) -> impl Future<Item = PostResponse, Error = FaunaError> + Send {
            let mut reference = Ref::instance(id);
            reference.set_class("posts");

            FAUNA
                .query(Delete::new(reference))
                .then(|result| Ok(PostResponse::from(result)))
        }

        #[get("/posts/:id")]
        #[content_type("application/json")]
        fn find(&self, id: String) -> impl Future<Item = PostResponse, Error = FaunaError> + Send {
            let mut reference = Ref::instance(id);
            reference.set_class("posts");

            FAUNA
                .query(Get::instance(reference))
                .then(|result| Ok(PostResponse::from(result)))
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
