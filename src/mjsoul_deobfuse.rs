pub fn decode_log_id(id: &str) -> String {
    const ZERO: u8 = b'0';
    const ALPHA: u8 = b'a';

    let mut ret = String::with_capacity(id.len());
    for (i, &code) in id.as_bytes().iter().enumerate() {
        let mut o = if (ZERO..ZERO + 10).contains(&code) {
            code - ZERO
        } else if (ALPHA..ALPHA + 26).contains(&code) {
            code - ALPHA + 10
        } else {
            ret.push(code as char);
            continue;
        };

        o = (o + 55 - i as u8) % 36;
        if o < 10 {
            ret.push((o + ZERO) as char)
        } else {
            ret.push((o + ALPHA - 10) as char)
        }
    }

    ret
}
