pub async fn quote_encrypt_unquote(data: &mut AsyncRead) -> Vec<u8> {
    let mut pad = AsyncRead::new(vec![4; 32]); // chosen by fair dice roll
    let data = await!(data.read_to_end());
    let pad = await!(pad.read_to_end());
    data.into_iter().zip(pad).map(|(a, b)| a ^ b).collect()
}
