existential type ReadToEnd<'a>: Future<Output = Vec<u8>> + 'a;

fn read_to_end(read: &mut AsyncRead) -> ReadToEnd<'_> {
    read.read_to_end()
}

struct ManualGenerator<'a> {
    state: i32,
    data_1: MaybeUninit<&'a mut AsyncRead>,
    pad_1: MaybeUninit<AsyncRead>,
    pinned_1: MaybeUninit<ReadToEnd<'a>>,
    data_2: MaybeUninit<Vec<u8>>,
    pinned_2: MaybeUninit<ReadToEnd<'a>>,
}

impl<'a> Generator for ManualGenerator<'a> {
    type Yield = ();
    type Return = Vec<u8>;

    unsafe fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
        match self.state {
            0 => {
                // one-time-pad chosen by fair dice roll
                self.pad_1.set(AsyncRead::new(vec![4; 32]));
                self.pinned_1
                    .set(read_to_end(&mut *self.data_1.as_mut_ptr()));
                self.state = 1;
                self.resume()
            }
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
            -1 => panic!("ManualGenerator polled after completion"),
            -2 => panic!("ManualGenerator polled after dropped"),
            _ => panic!("ManualGenerator polled with invalid state"),
        }
    }
}

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
