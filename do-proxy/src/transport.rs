use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub(crate) enum RequestTransport<Init, Request> {
    InitWithRequest {
        init: Init,
        request: Request,
    },
    Init {
        init: Init,
    },
    Request {
        request: Request,
    },
    #[doc(hidden)]
    #[serde(skip)]
    Empty,
}

impl<Init, Request> RequestTransport<Init, Request> {
    pub fn take_init(&mut self) -> Option<Init> {
        let this = std::mem::replace(self, RequestTransport::Empty);

        match this {
            RequestTransport::Init { init } => {
                *self = RequestTransport::Empty;
                Some(init)
            }
            RequestTransport::Request { request } => {
                *self = RequestTransport::Request { request };
                None
            }
            RequestTransport::InitWithRequest { init, request } => {
                *self = RequestTransport::Request { request };
                Some(init)
            }
            RequestTransport::Empty => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub(crate) enum ResponseTransport<Response, Error> {
    Response { response: Response },
    Error { error: Error },
    Initialized,
}
