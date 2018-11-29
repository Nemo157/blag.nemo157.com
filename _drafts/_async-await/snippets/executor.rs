pub fn block_on<F: Future>(mut future: F) -> F::Output {
    use core::{
        pin::Pin,
        ptr::NonNull,
        task::{LocalWaker, Poll, UnsafeWake, Waker},
    };

    struct NoWake;

    impl NoWake {
        fn local_waker() -> LocalWaker {
            // Safety: all references to NoWake are never
            // dereferenced
            unsafe {
                LocalWaker::new(
                    NonNull::<NoWake>::dangling(),
                )
            }
        }
    }

    unsafe impl UnsafeWake for NoWake {
        unsafe fn clone_raw(&self) -> Waker {
            NoWake::local_waker().into_waker()
        }
        unsafe fn drop_raw(&self) {}
        unsafe fn wake(&self) {}
    }

    let lw = NoWake::local_waker();
    loop {
        // Safety: `future` is a local variable which is
        // only ever used in this pinned reference
        match unsafe { Pin::new_unchecked(&mut future) }
            .poll(&lw)
        {
            Poll::Ready(value) => break value,
            Poll::Pending => continue,
        }
    }
}
