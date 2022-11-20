use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
};

use worker::Stub;

use crate::{
    transport::{RequestTransport, ResponseTransport},
    CrateOrObjectError, DoProxy,
};

/// A wrapper around a [`worker::Stub`] that provides a builder interface for
/// sending requests. This is the only way to send requests to a durable object
/// in a type-safe way.
///
/// Create proxies using the [`crate::EnvExt`] trait.
///
/// The `Builder` type returned by [`Proxy::send`] and [`Proxy::init`]
/// implements [`std::future::IntoFuture`]. This means, you must use `.await` to
/// actually send the request.
pub struct Proxy<O> {
    stub: Stub,
    _phantom: PhantomData<O>,
}

impl<O: DoProxy> Proxy<O> {
    pub(crate) fn new(stub: Stub) -> Self {
        Self {
            stub,
            _phantom: PhantomData,
        }
    }

    /// Send a request to the durable object. You must await this future to
    /// # Example
    ///
    /// ```ignore
    /// let resp = proxy.send(Command::GetBirthday).await?;
    /// ```
    #[must_use = "you must await this future to send the request"]
    pub fn send(&self, request: O::Request) -> Builder<'_, O, Send> {
        Builder::new(&self.stub).send(request)
    }

    /// Send a request to the durable object. You can immediately `await` the
    /// result of this method to initialize the object, or you can chain a
    /// `.and_send()` call to have the object handle a request after it is
    /// initialized.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Initialize a person object, no response is expected.
    /// proxy.init(Person::new("Bob")).await?;
    ///
    /// // or
    /// let resp = proxy.init(Person::new("Bob")).and_send(Command::GetBirthday).await?;
    /// ```
    #[must_use = "you must await this future to send the request"]
    pub fn init(&self, init: O::Init) -> Builder<'_, O, WithInit> {
        Builder::new(&self.stub).init(init)
    }
}

pub struct Builder<'s, O: DoProxy, State> {
    stub: &'s Stub,
    request: RequestTransport<O::Init, O::Request>,
    _phantom: PhantomData<State>,
}

pub struct New;
pub struct WithInit;
pub struct Send;

impl<'s, O: DoProxy> Builder<'s, O, New> {
    pub fn new(stub: &'s Stub) -> Self {
        Self {
            stub,
            request: RequestTransport::Empty,
            _phantom: PhantomData,
        }
    }
}

impl<'s, O: DoProxy> Builder<'s, O, New> {
    pub fn send(self, request: O::Request) -> Builder<'s, O, Send> {
        Builder {
            stub: self.stub,
            request: RequestTransport::Request { request },
            _phantom: PhantomData,
        }
    }

    pub fn init(self, init: O::Init) -> Builder<'s, O, WithInit> {
        Builder {
            stub: self.stub,
            request: RequestTransport::Init { init },
            _phantom: PhantomData,
        }
    }
}

impl<'s, O: DoProxy> Builder<'s, O, WithInit> {
    pub fn and_send(mut self, request: O::Request) -> Builder<'s, O, Send> {
        Builder {
            stub: self.stub,
            request: RequestTransport::InitWithRequest {
                init: self.request.take_init().unwrap(),
                request,
            },
            _phantom: PhantomData,
        }
    }
}

impl<'s, O: DoProxy> Builder<'s, O, Send> {
    async fn run(self) -> Result<O::Response, CrateOrObjectError<O::Error>> {
        match send_to_stub::<O>(self.stub, self.request).await {
            Ok(response) => match response {
                ResponseTransport::Response { response } => Ok(response),
                ResponseTransport::Error { error } => Err(CrateOrObjectError::Object(error)),
                ResponseTransport::Initialized => Err(crate::Error::ExpectedObjectResponse.into()),
            },
            Err(error) => Err(error.into()),
        }
    }
}

impl<'s, O: DoProxy> Builder<'s, O, WithInit> {
    async fn run(self) -> Result<Result<(), O::Error>, crate::Error> {
        match send_to_stub::<O>(self.stub, self.request).await {
            Ok(response) => match response {
                ResponseTransport::Initialized => Ok(Ok(())),
                ResponseTransport::Response { .. } => Err(crate::Error::ExpectedObjectInitialized),
                ResponseTransport::Error { error } => Ok(Err(error)),
            },
            Err(error) => Err(error),
        }
    }
}

impl<'s, O: DoProxy + 's> IntoFuture for Builder<'s, O, Send> {
    type Output = Result<O::Response, CrateOrObjectError<O::Error>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 's>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.run().await })
    }
}

impl<'s, O: DoProxy + 's> IntoFuture for Builder<'s, O, WithInit> {
    type Output = Result<Result<(), O::Error>, crate::Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 's>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.run().await })
    }
}

async fn send_to_stub<O: DoProxy>(
    stub: &Stub,
    req: RequestTransport<O::Init, O::Request>,
) -> Result<ResponseTransport<O::Response, O::Error>, crate::Error> {
    let json = serde_json::to_string(&req)?;

    let mut request_init = worker::RequestInit::new();
    request_init
        .with_method(worker::Method::Post)
        .with_body(Some(json.into()));

    let request =
        worker::Request::new_with_init(&format!("http://{}/", O::BINDING), &request_init)?;
    let response: ResponseTransport<O::Response, O::Error> =
        stub.fetch_with_request(request).await?.json().await?;

    Ok(response)
}
