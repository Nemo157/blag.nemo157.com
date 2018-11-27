---
layout: post
title:  "TODO: Rust Async Await"
date:   2018-11-27 23:38:06 +0100
---

## Intro

## The Setup

First, to provide a nice self-contained example I need to get some setup out of
the way. In normal async/await in Rust you will bring in the [`futures`][]
library to handle most of the pieces I'm about to mention; but to be able to
provide links to running playgrounds of these examples (and so that you can see
all the gristle in these here sausages) I'm going to avoid that. 

Also, I'm going to assume complete knowledge of soon to be stable Rust 2018 and
the basic futures API (including pinning). I will endeavour to explain all the
other nightly features being used, but I have been living in nightly-only for
far too long now and may forget some.

### The "Async IO"

To provide a slightly more realistic example I will use the following "trait"
for reading in data:

{% include code.md code="async-read.rs" %}

This can't be a real trait because it requires generic associated types to
declare the return value of `read_to_end`, so wherever you see `AsyncRead` in
the later function signatures just imagine `impl AsyncRead`.

You may also note a lack of errors. This is because adding in the additional
error handling paths in the last step is a _lot_ of work and I'm lazy. They
don't really show anything new either, just more matching and more states to
implement to handle the different execution paths.

### The "Executor"

To run a `Future` to completion you require an executor to run it on. This is
the simplest ever executor that simply spins on a single future until it
completes:

TODO: no-block.rs

## The `async`/`await!` implementation

Now, here's the super simple `async fn` we're going to be expanding. This
function takes in a reference to some async IO, constructs a handle to some
"random" one-time-pad, waits for both to complete, then XORs the data and pad
together to secure the data. It may seem simple here, but once we get to the
final stage you're going to be glad I chose something so simple.

TODO: async-example.rs

## Expanding `async`/`await!`

The first step on our journey into madness is to simply expand the `async fn`
into a normal function. This has three parts to it:

 1. Expanding the `await!` macro
 2. Rewriting the function signature
 3. Expanding the body

### But first

Before doing the actual `async`/`await!` expansion I want to slightly rewrite
the function from before. This will be a functionally equivalent function, but
by pulling out a few temporary variables the control flow between the different
transforms will be easier to follow. Mostly, having `await!` inside other
expressions will greatly complicate the upcoming generator transform.

TODO: async-example-2.rs

### Expanding the `await!` macro

Currently `await!` is simply a normal macro [defined in `std`][std::await]. This
is unlikely to last, there are some requirements on it that I believe will
necessitate it moving into the compiler, but it makes things slightly simpler
here for now. We can expand this macro while still leaving the rest of the
`async fn` alone and still get something that compiles. Note the
`poll_with_tls_waker` function introduced here, I'll come back to it later.

[std::await]: https://doc.rust-lang.org/nightly/std/macro.await.html

TODO: async-example-3.rs

### Rewriting the function signature

`async fn` does some slightly funky things to the function signature. The main
thing is just taking the return value (`R`) and wrapping it into `impl
Future<Output = R>`, then the lifetimes of all arguments are bound into this
return value (TODO: work out phrase here instead of "bound into"). Currently if
you have a function taking multiple references you have to give it a single
named lifetime for all those references to use, but I believe the intention is
for this to automatically work in the future.

After re-writing the signature the body of the function can be wrapped in
an `async move { ... }` block to keep everything compiling with the exact same
semantics as before:

TODO: async-example-4.rs

### Expanding the body

Finally to remove all `async` syntax from the function we can expand the `async
move { ... }` block into a generator and package this into a wrapper `Future`
with `from_generator` from `std::future`.

`std::future::from_generator` is a counterpart to the `poll_with_tls_waker`
function mentioned earlier, these _must_ be used together, the `Future` created
by `from_generator` will place the `&LocalWaker` passed in to `Future::poll`
into thread local storage, `poll_with_tls_waker` will then retrieve this to pass
in to sub-futures that are being `await!`ed on. 

TODO: async-example-5.rs