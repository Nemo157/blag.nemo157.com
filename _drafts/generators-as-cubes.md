---
layout: post
title:  "Generators as Cubes"
subtitle:  "Isomorphic Async"
excerpt_separator: <!--more-->
---

TODO: Background

```rust
trait Generator<Resume> {
    type Yield;
    type Complete;

    fn resume(self: Pin<&mut Self>, args: Resume) -> GeneratorState<Yield, Resume>;
}
```

You can imagine the `Generator` trait as covering a three dimensional space,
each concrete implementation is then a point in this space (a generic
implementation is sort of a line segment, if you rearrange the `Resume` axis to
have the types adjacent).

```rust
impl<G> Future for G where G: for<'a> Generator<(&'a Waker,), Yield = ()> {
    type Output = G::Complete;
}

impl<G> Stream for AsAsync<G> where G: for<'a> Generator<(&'a Waker,), Yield: IsPoll, Complete = ()> {
    type Item = G::Yield::Ready;
}


For the async types (`Future` and `Stream`) the `Resume` argument is then
constrained to be a single type (`&Waker`), so these types inhabit a single
plane across the `Yield` ⨯ `Complete` axes. `Future` then further limits `Yield
= Poll::Pending` so is a single line along the `Complete` axis, while `Stream` limits `Complete = Poll::Ready(None)`
