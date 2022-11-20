use crate::{DoProxy, Proxy};

/// The [`EnvExt`] trait makes it easy to create proxies from a [`worker::Env`].
///
/// Defines multiple methods on the [`workers::Env`] type to make it easier to
/// create proxies.
///
/// # Example
///
/// ```ignore
/// #[worker::event(fetch, respond_with_errors)]
/// pub async fn main(mut req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
///     let url = req.url()?;
///     let do_name = url.path();
///
///     let command = req.json().await?;
///     
///     // Easily access an `Inserter` object with the given name.
///     let inserter = env.obj::<Inserter>(do_name)?;
///     let resp = inserter.send(command).await??;
///
///     Response::from_json(&resp)
/// }
/// ```
pub trait EnvExt {
    /// Get a proxy to a durable object with the given name.
    ///
    /// ```ignore
    /// env.obj::<Inserter>("inserter_for_fisher")?;
    /// ```
    fn obj<Obj>(&self, name: &str) -> Result<Proxy<Obj>, worker::Error>
    where
        Obj: DoProxy;

    /// Get a proxy to a durable object with the given hex ID.
    ///
    /// ```ignore
    /// env.obj::<Inserter>("<long_hex_string>")?;
    /// ```
    fn obj_from_id<Obj>(&self, id: &str) -> Result<Proxy<Obj>, worker::Error>
    where
        Obj: DoProxy;

    /// Get a unique proxy to a durable object.
    fn unique_obj<Obj>(&self) -> Result<Proxy<Obj>, worker::Error>
    where
        Obj: DoProxy;
}

impl EnvExt for worker::Env {
    fn obj<Obj>(&self, name: &str) -> Result<Proxy<Obj>, worker::Error>
    where
        Obj: DoProxy,
    {
        let binding = self.durable_object(Obj::BINDING)?;
        let obj = binding.id_from_name(name)?;
        let stub = obj.get_stub()?;

        Ok(Proxy::new(stub))
    }

    fn obj_from_id<Obj>(&self, id: &str) -> Result<Proxy<Obj>, worker::Error>
    where
        Obj: DoProxy,
    {
        let binding = self.durable_object(Obj::BINDING)?;
        let obj = binding.id_from_string(id)?;
        let stub = obj.get_stub()?;

        Ok(Proxy::new(stub))
    }

    fn unique_obj<Obj>(&self) -> Result<Proxy<Obj>, worker::Error>
    where
        Obj: DoProxy,
    {
        let binding = self.durable_object(Obj::BINDING)?;
        let obj = binding.unique_id()?;
        let stub = obj.get_stub()?;

        Ok(Proxy::new(stub))
    }
}
