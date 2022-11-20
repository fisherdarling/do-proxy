# do-proxy

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/do-proxy.svg
[crates-url]: https://crates.io/crates/do-proxy
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/fisherdarling/do-proxy/blob/master/LICENSE

A library for writing type-safe [Durable
Objects](https://developers.cloudflare.com/workers/learning/using-durable-objects/)
(DOs) in Rust.

With `do-proxy` you can:
- Easily write type-safe APIs for Durable Objects.
- Abstract over `fetch`, `alarm` and request-response glue code.

## Overview

do-proxy provides a core trait `DoProxy` that abstracts over ser/de request
response code, object initalization and loading, and Error handling glue code.

After a struct implements `DoProxy`, the macro `do_proxy!` creates the
[workers-rs](https://github.com/cloudflare/workers-rs)' `#[DurableObject]`
struct which ends up generating the final object.

## Object Lifecycle

This library provides two separate `Request` type. A normal `Request` which is
what the durable object will be sent 99% of the time and an `Init` type, which is
optionally sent to initialize the object.

For example, lets say we have a `Person` object. The struct might look like:

```rust
struct Person {
    birthday: DateTime<Utc>,
    name: String
}

impl DoProxy for Person {
    // ...
}

do_proxy!(Person, PersonObject);
```

The `birthday` and `name` fields are non-optional and required. However, when
constructing a durable object with `new` in Rust or `constructor` in TypeScript,
the only information you get is the `State` and `Storage`. So when you're
loading the `Person` object for the first time, you'll have to use bogus values
for `name` and `birthday` because they haven't been set yet.

The request that prompted the creation of the object _will likely_ be some kind
of "create" command which sets `birthday` and `name` to something realistic. But
that's not guaranteed.

What if the person receives a command `CalculateNextBirthday` before its been
created? Now you need to explicitly check that those values aren't bogus _or_
wrap everything in an `Option`. Both of those options are either prone to errors
(bogus value) or unergonomic (`Option`). 

To solve this, `do-proxy` has two functions that are used to construct an
object.

- `init`: crates and saves all information necessary to construct an object in
  `load_from_storage`. When a user of the library sends a request to the
  `Person` object, they'll be able to optionally add `DoProxy::Init` data. If
  that data is present, `init` will be called _before_ `load_from_storage`.
- `load_from_storage`: loads an object from storage. If the object is missing
  fields or hasn't been initialized, this function should error.

In the following example, we send both initialization information along with a
command. The object is first initialized _and then_ the command is handled:

```rust
let proxy = env.obj::<Person>("bob@buzz.com");
let resp = proxy
    .init(Person::new("Bob", bobs_birthday))
    .and_send(Command::CalculateNextBirthday).await?;
```

If you know that the object must be initialized or can't create initialization
information, you can just send the command:

```rust
let proxy = env.obj::<Person>("bob@buzz.com");
let resp = proxy.send(Command::CalculateNextBirthday).await?;
```

This approach lets you avoid options, bogus values _and_ the `init` function
is `async`.

## Examples

The crates under [./examples](./examples/) act as examples for the library, and
in the future, they act as fully-fledged copy+paste building blocks for
distributed systems.

Each crate has a [hurl](https://hurl.dev/) script that show how the wrapping
worker can be queried.

- [`inserter`](./examples/inserter/): An InserterObject responds to a simple
  KV-like API for getting, inserting and deleting KV pairs in its storage.
  Example:

```sh
POST http://localhost:8787/test_do
{
    "insert": {
        "key": "hello",
        "value": "world!"
    }
}

POST http://localhost:8787/test_do
{
    "get": {
        "key": "hello"
    }
}

# returns { "value": "world!" }
```

