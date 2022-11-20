mod inserter;

use self::inserter::Inserter;

use do_proxy::EnvExt;
use worker::*;

/// A simple pass-through worker that forwards commands to the given durable
/// object and returns its response.
#[worker::event(fetch, respond_with_errors)]
pub async fn main(mut req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let url = req.url()?;
    let do_name = url.path();

    // Deserialize a command from the request
    let command = req.json().await?;

    // Easily access an `Inserter` object with the given name.
    let inserter = env.obj::<Inserter>(do_name)?;

    // Send the command to the object.
    let resp = inserter.send(command).await?;

    Response::from_json(&resp)
}
