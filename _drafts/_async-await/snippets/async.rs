pub async fn quote_encrypt_unquote(
    data: &mut AsyncRead,
) -> Vec<u8> {
    // one-time-pad chosen by fair dice roll
    let mut pad = AsyncRead::new(vec![4; 32]);
    await!(data.read_to_end())
        .into_iter()
        .zip(await!(pad.read_to_end()))
        .map(|(a, b)| a ^ b)
        .collect()
}
