#![allow(unused)]

use std::{error::Error, marker::PhantomData};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use worker::{Env, State, Stub};

use crate::transport::{RequestTransport, ResponseTransport};

/// A request sent to an object.
pub enum ProxiedRequest<R> {
    Fetch(R),
    Alarm,
}

/// The [`DoProxy`] trait is the main interface of this library. Implement this
/// trait for a type you want to make into a durable object and automatically
/// get many helper methods for interacting with it.
///
/// After implementing this trait, use the macro `do_proxy!` to generate the
/// workers-rs [`worker::DurableObject`] glue code.
///
/// See the crates under `examples/*` for example implementations.
#[async_trait(?Send)]
pub trait DoProxy
where
    Self: Sized,
{
    /// The Durable Object's binding. Must be the same as the one written in
    /// your `wrangler.toml`. For example, `INSERTER_OBJECT`.
    const BINDING: &'static str;

    /// The initialization data that will be passed to the the object when it is
    /// first created. This should be used to set data that is expected to
    /// always be available when the object loads. For example, the first time a
    /// `Person` object is created, it should be initialized with the person's
    /// name. This way when `load_from_storage` is called, we can expect a
    /// stored `name` field to always be present. If it is not present, then the
    /// object has not yet been initialized and `load_from_storage` should
    /// error.
    ///
    /// # Example
    ///
    /// ```ignore
    /// struct NewPerson {
    ///     name: String,
    ///     birthday: DateTime<Utc>,
    /// }
    /// ```
    type Init: Serialize + DeserializeOwned + 'static;
    /// The request type that will be sent to the object. This is generally an
    /// enum of all of the different "commands" that the object can handle.
    ///
    /// # Example
    ///
    /// ```ignore
    /// enum PersonRequest {
    ///     GetAge,
    ///     GetNextBirthday,
    ///     GetName,
    /// }
    /// ```
    type Request: Serialize + DeserializeOwned + 'static;
    /// The response type that will be sent back from the object This is generally
    /// an enum of all of the different "responses" that the object can send.
    ///
    /// Note, types like `Option<serde_json::Value>` will work!
    ///
    /// # Example
    ///
    /// ```ignore
    /// enum PersonResponse {
    ///     Age(usize),
    ///     Birthday(chrono::DateTime<chrono::Utc>),
    ///     GetName(String),
    /// }
    /// ```
    type Response: Serialize + DeserializeOwned + 'static;

    /// The error type that will be returned from the object. This lets users
    /// cleanly (kind of) pass errors from the object back to the caller.
    ///
    /// # Example
    ///
    /// ```ignore
    /// enum PersonError {
    ///     NotYetBorn,
    /// }
    /// ```
    type Error: Serialize + DeserializeOwned + Error;

    /// Called if the object is sent an `init` request. This function may be
    /// called multiple times and implemeting it is _optional_.
    async fn init(ctx: &mut Ctx, init: Self::Init) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when the object is first loaded into memory. This function is
    /// generally called only once when the Durable Object first receives a
    /// request. If the object is evicted from memory and then later receives a
    /// request, this function will be called again.
    async fn load_from_storage(ctx: &mut Ctx) -> Result<Self, Self::Error>;

    /// Called when the object receives a fetch request or an alarm. This is
    /// generally where you would match on [`Self::Request`] and call the
    /// appropriate function.
    async fn handle(
        &mut self,
        ctx: &mut Ctx,
        req: ProxiedRequest<Self::Request>,
    ) -> Result<Self::Response, Self::Error>;

    /// This function wraps the `handle` function and handles the boilerplate of
    /// caching, converting between the different transport types, and error
    /// handling.
    ///
    /// You should never implement this function, however you can if you need
    /// to.
    async fn run_request(
        cached_proxy: &mut Option<Self>,
        ctx: &mut Ctx,
        req: Option<worker::Request>,
    ) -> worker::Result<worker::Response> {
        enum TransportOrAlarm<Init, Request> {
            Transport(RequestTransport<Init, Request>),
            Alarm,
        }

        let mut transport_or_alarm: TransportOrAlarm<Self::Init, Self::Request> = match req {
            Some(mut req) => TransportOrAlarm::Transport(req.json().await?),
            None => TransportOrAlarm::Alarm,
        };

        let mut proxy = match cached_proxy.take() {
            Some(proxy) => proxy,
            None => {
                if let Some(init) = match transport_or_alarm {
                    TransportOrAlarm::Transport(ref mut transport) => transport.take_init(),
                    TransportOrAlarm::Alarm => None,
                } {
                    Self::init(ctx, init).await.map_err(|e| e.to_string())?;
                    Self::load_from_storage(ctx)
                        .await
                        .map_err(|e| e.to_string())?
                } else {
                    Self::load_from_storage(ctx)
                        .await
                        .map_err(|e| e.to_string())?
                }
            }
        };

        let response = match transport_or_alarm {
            TransportOrAlarm::Transport(RequestTransport::Request { request }) => {
                match proxy.handle(ctx, ProxiedRequest::Fetch(request)).await {
                    Ok(response) => ResponseTransport::Response { response },
                    Err(error) => ResponseTransport::Error { error },
                }
            }
            TransportOrAlarm::Alarm => match proxy.handle(ctx, ProxiedRequest::Alarm).await {
                Ok(response) => ResponseTransport::Response { response },
                Err(error) => ResponseTransport::Error { error },
            },
            TransportOrAlarm::Transport(RequestTransport::Empty) => ResponseTransport::Initialized,
            _ => {
                unreachable!("RequestTransport::Init and RequestTransport::InitWithRequest should have been handled by the match arm above");
            }
        };

        *cached_proxy = Some(proxy);
        worker::Response::from_json(&response)
    }
}

/// The context that is passed to the object's `init`, `load_from_storage`, and `handle` functions.
///
/// Wraps [`worker::State`] and [`worker::Env`].
pub struct Ctx<'s> {
    pub state: &'s State,
    pub env: &'s Env,
}

impl<'s> Ctx<'s> {
    pub fn new(state: &'s State, env: &'s Env) -> Self {
        Self { state, env }
    }
}
