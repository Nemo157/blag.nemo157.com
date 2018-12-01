#![feature(async_await, await_macro, futures_api, pin)]

pub mod io {
    use core::future::Future;

    pub struct AsyncRead(Vec<u8>);

    impl AsyncRead {
        pub fn new(data: Vec<u8>) -> AsyncRead {
            AsyncRead(data)
        }

        pub fn read_to_end(&mut self) -> impl Future<Output = Vec<u8>> + '_ {
            async move { self.0.clone() }
        }
    }
}

pub mod executor {
    use core::{
        future::Future,
        pin::Pin,
        ptr::NonNull,
        task::{LocalWaker, Poll, UnsafeWake, Waker},
    };

    struct NoWake;
    impl NoWake {
        fn local_waker() -> LocalWaker {
            // Safety: all references to NoWake are never dereferenced
            unsafe { LocalWaker::new(NonNull::<NoWake>::dangling()) }
        }
    }
    unsafe impl UnsafeWake for NoWake {
        unsafe fn clone_raw(&self) -> Waker {
            NoWake::local_waker().into_waker()
        }
        unsafe fn drop_raw(&self) {}
        unsafe fn wake(&self) {}
    }

    pub fn block_on<F: Future>(mut future: F) -> F::Output {
        let lw = NoWake::local_waker();
        loop {
            // Safety: `future` is a local variable which is only ever used in this
            // pinned reference
            match unsafe { Pin::new_unchecked(&mut future) }.poll(&lw) {
                Poll::Ready(value) => break value,
                Poll::Pending => continue,
            }
        }
    }
}

use self::io::AsyncRead;

pub async fn quote_encrypt_unquote(data: &mut AsyncRead) -> Vec<u8> {
    // one-time-pad chosen by fair dice roll
    let mut pad = AsyncRead::new(vec![4; 32]);
    await!(data.read_to_end())
        .into_iter()
        .zip(await!(pad.read_to_end()))
        .map(|(a, b)| a ^ b)
        .collect()
}

fn main() {
    let mut data = AsyncRead::new("hello".into());
    let encrypted = executor::block_on(quote_encrypt_unquote(&mut data));
    println!("Encrypted: {}", core::str::from_utf8(&encrypted).unwrap());
}
