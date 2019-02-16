---
layout: post
title:  "Async Transform via Resume Arguments"
excerpt_separator: <!--more-->
---

TODO: Background

```rust
trait IsPoll {
    type Ready;

    fn into_poll(self) -> Poll<Self::Ready>;
}

impl<T> IsPoll for Poll<T> {
    type Ready = T;

    fn into_poll(self) -> Poll<Self::Ready> { self }
}

struct AsAsync<T>(T);
struct AsGenerator<T>(T);
```

```rust
impl<G> Future for AsAsync<G> where G: for<'a> Generator<(&'a Waker,), Yield = Poll<!>> {
    type Output = G::Complete;

    fn poll(self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        match self.0.resume((waker,)) {
            GeneratorState::Yield(Poll::Pending) => Poll::Pending,
            GeneratorState::Complete(output) => Poll::Ready(output),
        }
    }
}

impl<'a, F> Generator<(&'a Waker,)> for AsGenerator<F> where F: Future {
    type Yield = Poll<!>;
    type Complete = F::Output;

    fn resume(self: Pin<&mut Self>, (waker,): (&'a Waker,)) -> GeneratorState<Self::Yield, Self::Complete> {
        match self.0.poll(waker) {
            Poll::Pending => GeneratorState::Yield(Poll::Pending),
            Poll::Ready(output) => GeneratorState::Complete(output),
        }
    }
}

impl<G> Stream for AsAsync<G> where G: for<'a> Generator<(&'a Waker,), Yield: IsPoll, Complete = ()> {
    type Item = G::Yield::Ready;

    fn poll_next(self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        match self.0.resume((waker,)) {
            GeneratorState::Yield(item) => item.into_poll().map(Some),
            GeneratorState::Complete(()) => Poll::Ready(None),
        }
    }
}

impl<'a, S> Generator<(&'a Waker,)> for AsGenerator<S> where S: Stream {
    type Yield = Poll<S::Item>;
    type Complete = ();

    fn resume(self: Pin<&mut Self>, (waker,): (&'a Waker,)) -> GeneratorState<Self::Yield, Self::Complete> {
        match self.0.poll_next(waker) {
            Poll::Ready(Some(item)) => GeneratorState::Yield(Poll::Ready(item)),
            Poll::Pending => GeneratorState::Yield(Poll::Pending),
            Poll::Ready(None) => GeneratorState::Complete(()),
        }
    }
}
```

The `Future` implementation up there is slightly more complicated than it must
be, instead of using `Yield = ()` we're using the equivalent `Yield =
Poll<!>`. This is useful later on as it allows the `await` transform to be
identical between `Future` and `Stream`.

Also, ignore the lack of pin projections, lets just assume the compiler is
supporting `Pin` directly now somehow (the implementation is trivial but
tedious).


