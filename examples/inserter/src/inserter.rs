use do_proxy::{async_trait, do_proxy, DoProxy, ProxiedRequest};
use serde::{Deserialize, Serialize};

pub struct Inserter;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InserterRequest {
    Insert {
        key: String,
        value: serde_json::Value,
    },
    Get {
        key: String,
    },
    Delete {
        key: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InserterResponse {
    Inserted,
    Deleted,
    Value(Option<serde_json::Value>),
}

#[async_trait(?Send)]
impl DoProxy for Inserter {
    const BINDING: &'static str = "INSERTER_OBJECT";

    type Init = ();
    type Request = InserterRequest;
    type Response = InserterResponse;
    type Error = do_proxy::Error;

    async fn load_from_storage(_ctx: &mut do_proxy::Ctx) -> Result<Self, Self::Error> {
        Ok(Self)
    }

    async fn handle(
        &mut self,
        ctx: &mut do_proxy::Ctx,
        req: ProxiedRequest<Self::Request>,
    ) -> Result<Self::Response, Self::Error> {
        match req {
            ProxiedRequest::Fetch(req) => match req {
                InserterRequest::Insert { key, value } => {
                    ctx.state.storage().put(&key, &value).await?;
                    Ok(InserterResponse::Inserted)
                }
                InserterRequest::Get { key } => {
                    let value = ctx.state.storage().get(&key).await.ok();
                    Ok(InserterResponse::Value(value))
                }
                InserterRequest::Delete { key } => {
                    ctx.state.storage().delete(&key).await?;
                    Ok(InserterResponse::Deleted)
                }
            },
            ProxiedRequest::Alarm => unimplemented!("alarm"),
        }
    }
}

// Create the actual durable object.
do_proxy!(Inserter, InserterObject);
