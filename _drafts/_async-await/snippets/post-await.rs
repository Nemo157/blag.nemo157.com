pub async fn quote_encrypt_unquote(data: &mut AsyncRead) -> Vec<u8> {
    use std::future::poll_with_tls_waker;
    use core::{pin::Pin, task::Poll};

    let mut pad = AsyncRead::new(vec![4; 32]); // chosen by fair dice roll
    let data = {
        let mut pinned = data.read_to_end();
        loop {
            if let Poll::Ready(x) = poll_with_tls_waker(unsafe { Pin::new_unchecked(&mut pinned) }) {
                break x;
            }
            yield
        }
    };
    let pad = {
        let mut pinned = pad.read_to_end();
        loop {
            if let Poll::Ready(x) = poll_with_tls_waker(unsafe { Pin::new_unchecked(&mut pinned) }) {
                break x;
            }
            yield
        }
    };
    data.into_iter().zip(pad).map(|(a, b)| a ^ b).collect()
}
