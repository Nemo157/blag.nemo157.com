---
layout: post
title:  "TODO: Rust Async Await"
date:   2018-11-27 23:38:06 +0100
---

TODO: Intro

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

[`futures`]: https://github.com/rust-lang-nursery/futures-rs

### Nightly Features

There are only three nightly features needed for the basic setup here, these are
what I'm assuming complete knowledge of:

 * `async_await` to allow using async functions and blocks
 * `futures_api` to allow using the basic `core::future` and `core::task` APIs
 * `pin` to allow using the basic `core::pin` APIs

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

To run a `Future` to completion you require an executor to run it on. First
there is the most basic API for an executor that can run a single future to
completion:

```rust
pub fn block_on<F: Future>(mut future: F) -> F::Output;
```

Then, this is the simplest implementation of that executor that simply spins
polling the future until it completes (there is an equivalent executor that
requires less lines of code, and no unsafety, but it does require use of
`std::sync::Arc` and I'm attempting to use the least powerful implementation of
everything here):

{% include code.md code="executor.rs" %}

## The `async`/`await!` implementation

Now that we have the setup out of the way, here's the super simple `async fn`
we're going to be expanding. This function takes in a reference to some async
IO, constructs a handle to some "random" one-time-pad, waits for both to
complete, then XORs the data and pad together to secure the data. It may seem
simple here, but once we get to the final stage you're going to be glad I chose
something so simple.

{% include code.md code="async.rs" %}

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

{% include code.md code="async-split.rs" %}

### Expanding the `await!` macro

Currently `await!` is simply a normal macro [defined in `std`][std::await]. This
is unlikely to last, there are some requirements on it that I believe will
necessitate it moving into the compiler, but it makes things slightly simpler
here for now. We can expand this macro while still leaving the rest of the
`async fn` alone and still get something that compiles. Note the
`poll_with_tls_waker` function introduced here, I'll come back to it later.

[std::await]: https://doc.rust-lang.org/nightly/std/macro.await.html

{% include code.md code="post-await.rs" %}

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

{% include code.md code="post-async.rs" %}

### Expanding the body

Finally to remove all `async` syntax from the function we can expand the `async
move { ... }` block into a generator and package this into a wrapper `Future`
with `from_generator` from `std::future`.

`std::future::from_generator` is a counterpart to the `poll_with_tls_waker`
function mentioned earlier, these _must_ be used together, the `Future` created
by `from_generator` will place the `&LocalWaker` passed in to `Future::poll`
into thread local storage, `poll_with_tls_waker` will then retrieve this to pass
in to sub-futures that are being `await!`ed on. 

{% include code.md code="generator.rs" %}

## Expanding the generator

Now that we have finished the `async` transform we have the much more
complicated generator transform to apply. This transform is implemented in
`rustc`'s MIR layer, so has much less direct equivalence to anything we can
implement manually in Rust's surface language. I'm going to present a
manual implementation of a `Generator` that does the same thing as the one from
the previous snippet, using a sort of similar layout to what `rustc` would
generate, but since this is all very much unstable internals there's no
guarantee that `rustc` will continue to generate something similar to this.

Also, rather than attempting to transform piece by piece like the last section
I'm going to first present the entire transformed generator, then pull out
pieces to explain from it. The following code block simply replaces everything
inside the `from_generator` call in the previous block.

{% include code.md code="manual-generator.rs" %}

### `ReadToEnd`

```rust
existential type ReadToEnd<'a>: Future<Output = Vec<u8>> + 'a;

fn read_to_end(read: &mut AsyncRead) -> ReadToEnd<'_> {
    read.read_to_end()
}
```

The very first part of this code block is a little hack to give a name to an
unnameable type. If you look back at `AsyncRead::read_to_end` you'll see that
the return type was declared as `impl Future<Output = Vec<u8>> + '_`.
Unfortunately we cannot easily store this type into the fields of a struct, so
we have this little hack using the `existential_type` feature to give ourselves
a name for the type.

### `struct ManualGenerator`

```rust
struct ManualGenerator<'a> {
    state: i32,
    data_1: MaybeUninit<&'a mut AsyncRead>,
    pad_1: MaybeUninit<AsyncRead>,
    pinned_1: MaybeUninit<ReadToEnd<'a>>,
    data_2: MaybeUninit<Vec<u8>>,
    pinned_2: MaybeUninit<ReadToEnd<'a>>,
}
```

Next we have the environment definition for the generator. This includes a
`state` variable to keep track of which yield point we are currently at, along
with fields for the upvars that were moved into the environment (`data_1`) and
the variables that live across yield points (`pad_1`, `pinned_1`, `data_2` and
`pinned_2`). Note that any variables that are only alive _between_ yield points
are not stored in the environment, these will be normal variables on the stack
of the `resume` function (the final `pad` result variable, and the temporaries
used during polling and the final line).

Each of the fields are stored as `MaybeUninit` as some of them start
uninitialized and others will be dropped before the generator finishes. You
might already be able to notice one potential optimization here, `pinned_1` and
`pinned_2` contain the same type but have non-overlapping lifetimes. To keep
closer to the real transform I have kept these as separate fields, see
[rust-lang/rust#52924][] for more details.

[rust-lang/rust#52924]: https://github.com/rust-lang/rust/issues/52924

### State `0`

```rust
0 => {
    // one-time-pad chosen by fair dice roll
    self.pad_1.set(AsyncRead::new(vec![4; 32]));
    self.pinned_1
        .set(read_to_end(&mut *self.data_1.as_mut_ptr()));
    self.state = 1;
    self.resume()
}
```

Now we get into the meat of the generator. First we have the code running from
the start of the closure until the beginning of the first `loop`. We don't go
all the way to the yield point because it is a yield point at the end of a loop,
we need a state number representing the start of the loop so we can resume there
after yielding. In the real transform this would end with a `goto` to the start
of the loop, but it's easier in the Rust surface syntax to just recurse into
`resume` again (we can't stack overflow as we guarantee moving to a new state
before recursing and have a limited number of states).

This is a pretty straight-forward transform from the original code, just note
that we use `MaybeUninit::set` to store the variables into the environment
instead of `=` and call our hacky `AsyncRead::read_to_end` wrapper so that we
know the type of the result.

### State `1`

```rust
1 => {
    self.data_2.set(loop {
        if let Poll::Ready(x) = poll_with_tls_waker(
            Pin::new_unchecked(self.pinned_1.get_mut()),
        ) {
            break x;
        }
        return GeneratorState::Yielded(());
    });
    self.pinned_2
        .set(read_to_end(&mut *self.pad_1.as_mut_ptr()));
    ptr::drop_in_place(self.pinned_1.as_mut_ptr());
    self.state = 2;
    self.resume()
}
```

State `1` covers the body of the first loop and the small bit of code between it
and the start of the second loop. Here we can see the same sort of
straightforward transform as the initial state, with two new things to
highlight:

 1. The `yield;` statement is simply replaced with `return
    GeneratorState::Yielded(());`. Because the yield is at the end of the loop
    the next statement to execute after resuming is the start of the loop again
    so this is relatively simple. In a more complicated generator where there is
    a yield in the middle of a loop you would need to split the body across
    multiple states and bounce between them until complete.

 2. The `pinned` variable is dropped at the end of its containing block. We have
    to use `drop_in_place` here to avoid moving it before dropping, it doesn't
    matter for this variable but if we were awaiting some `!Unpin` futures here
    then we would cause unsoundness if they were moved before `Drop::drop` was
    called.

### State `2`

```rust
2 => {
    let pad_2 = loop {
        if let Poll::Ready(x) = poll_with_tls_waker(
            Pin::new_unchecked(self.pinned_2.get_mut()),
        ) {
            break x;
        }
        return GeneratorState::Yielded(());
    };
    let result = ptr::read(self.data_2.as_mut_ptr())
        .into_iter()
        .zip(pad_2)
        .map(|(a, b)| a ^ b)
        .collect();
    ptr::drop_in_place(self.pinned_2.as_mut_ptr());
    ptr::drop_in_place(self.pad_1.as_mut_ptr());
    ptr::drop_in_place(self.data_1.as_mut_ptr());
    self.state = -1;
    GeneratorState::Complete(result)
}
```

Another straightforward transform for the body of the second loop, final
statement, and implicit drops of all the closure variables. This is the first
time we use a variable by value, we have to use `ptr::read` to move the final
`data` variable out to call `into_iter` on it, since we have passed ownership of
it off to `Vec::into_iter` we don't drop this variable, but do drop everything
else remaining alive before returning the final result.

### Error states

```rust
-1 => panic!("ManualGenerator polled after completion"),
-2 => panic!("ManualGenerator polled after dropped"),
_ => panic!("ManualGenerator polled with invalid state"),
```

We reserve two states for all generators: the generator has completed
successfully and the generator has been dropped. A generator should never be
polled after either of these states is reached, and should never be able to get
into any state we aren't otherwise handling, so we panic if these are reached.

### `Drop`

```rust
impl<'a> Drop for ManualGenerator<'a> {
    fn drop(&mut self) {
        match self.state {
            0 => unsafe {
                ptr::drop_in_place(self.data_1.as_mut_ptr());
            },
            1 => unsafe {
                ptr::drop_in_place(self.pinned_1.as_mut_ptr());
                ptr::drop_in_place(self.pad_1.as_mut_ptr());
                ptr::drop_in_place(self.data_1.as_mut_ptr());
            },
            2 => unsafe {
                ptr::drop_in_place(self.pinned_2.as_mut_ptr());
                ptr::drop_in_place(self.data_2.as_mut_ptr());
                ptr::drop_in_place(self.pad_1.as_mut_ptr());
                ptr::drop_in_place(self.data_1.as_mut_ptr());
            },
            -1 => { /* Everything already dropped in resume */ }
            -2 => panic!("ManualGenerator dropped twice"),
            _ => panic!("ManualGenerator dropped with invalid state"),
        }
        self.state = -2;
    }
}
```

Since we are using `MaybeUninit` for the fields we also need to implement `Drop`
for our generator manually, the compiler generated drop glue will not do
anything. We need to then check what state we are in and drop all the
variables that are still alive, before finally marking ourselves as dropped.

### `ManualGenerator::new`

```rust
impl<'a> ManualGenerator<'a> {
    fn new(data: &'a mut AsyncRead) -> Self {
        ManualGenerator {
            state: 0,
            data_1: MaybeUninit::new(data),
            pad_1: MaybeUninit::uninitialized(),
            pinned_1: MaybeUninit::uninitialized(),
            data_2: MaybeUninit::uninitialized(),
            pinned_2: MaybeUninit::uninitialized(),
        }
    }
}
```

Finally we need a way to construct the generator from the upvars, this doesn't
really get pulled out to a function in the real transform, but it's easier to
see what's happening if it is here.
