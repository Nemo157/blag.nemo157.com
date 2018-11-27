pub async fn quote_encrypt_unquote(data: &mut AsyncRead) -> Vec<u8> {
    let mut pad = AsyncRead::new(vec![4; 32]); // chosen by fair dice roll
    await!(data.read_to_end())
        .into_iter()
        .zip(await!(pad.read_to_end()))
        .map(|(a, b)| a ^ b)
        .collect()
}
