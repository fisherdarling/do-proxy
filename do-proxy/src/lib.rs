//! # do-proxy
//! A library for writing type-safe [Durable
//! Objects](https://developers.cloudflare.com/workers/learning/using-durable-objects/)
//! (DOs) in Rust.
//!
//! With `do-proxy` you can:
//! - Easily write type-safe APIs for Durable Objects.
//! - Abstract over `fetch`, `alarm` and request-response glue code.
//!
//! ## Overview
//!
//! do-proxy provides a core trait [`DoProxy`] that abstracts over ser/de request
//! response code, object initalization and loading, and Error handling glue code.
//!
//! After a struct implements [`DoProxy`], the macro [`do_proxy!`] creates the
//! [workers-rs](https://github.com/cloudflare/workers-rs)' `#[DurableObject]`
//! struct which ends up generating the final object.
//!
//! See [`DoProxy`] for more details.
mod env_ext;
mod error;
mod macros;
mod proxy;
mod proxy_trait;
mod transport;

pub use self::{
    env_ext::EnvExt,
    error::{CrateOrObjectError, Error},
    proxy::Proxy,
    proxy_trait::{Ctx, DoProxy, ProxiedRequest},
};

pub use ::async_trait::async_trait;
pub use ::paste;
pub use ::worker;
