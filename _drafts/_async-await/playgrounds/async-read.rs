#![feature(async_await, futures_api)]

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

fn main() {
    // Not enough pieces to give a decent example
    let encrypted = [108, 97, 104, 104, 107];
    println!("Encrypted: {}", core::str::from_utf8(&encrypted).unwrap());
}
