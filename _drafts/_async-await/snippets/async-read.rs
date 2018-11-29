pub struct AsyncRead(Vec<u8>);

impl AsyncRead {
    pub fn new(data: Vec<u8>) -> AsyncRead {
        AsyncRead(data)
    }

    pub fn read_to_end(
        &mut self,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        async move self.0.clone()
    }
}
