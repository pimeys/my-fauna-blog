use faunadb::{client::Response, error::Error, FaunaResult};
use serde_json::{json, Value};

#[derive(Debug, Response)]
#[web(status = "201")]
pub struct HttpCreated {
    #[web(header)]
    pub location: String,
}

#[derive(Debug, Response)]
#[web(either)]
pub enum HttpResponse {
    Data(Value),
    Error(HttpError),
}

#[derive(Debug, Response)]
pub struct HttpError {
    #[web(status)]
    status: u16,
}

impl From<FaunaResult<Response>> for HttpResponse {
    fn from(result: FaunaResult<Response>) -> Self {
        match result {
            Ok(resp) => {
                let res = resp.resource;

                let payload = json!({
                    "id": res.get_reference().unwrap().id,
                    "title": res["data"]["title"],
                    "age_limit": res["data"]["age_limit"],
                });

                HttpResponse::Data(payload)
            }
            Err(Error::NotFound(_)) => HttpResponse::Error(HttpError { status: 404 }),
            Err(e) => {
                error!("FATAL ERROR: {:?}", e);
                HttpResponse::Error(HttpError { status: 500 })
            }
        }
    }
}
