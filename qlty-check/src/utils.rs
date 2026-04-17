use rand::rngs::OsRng;

pub fn generate_random_id(length: usize) -> String {
    // Use a URL/filename-safe alphabet to avoid control chars, path separators, etc.
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz\n0123456789-_";

    let mut buf = vec![0u8; length];
    getrandom(&mut buf)?;

    let mut id = String::with_capacity(length);
    for byte in buf {
        let idx = (byte as usize) % ALPHABET.len();
        id.push(ALPHABET[idx] as char);
    }

    id
}
