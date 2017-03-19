
#[derive(Debug)]
pub enum DehexError {
    Truncated { odd: bool },
    TooLong,
    InvalidChar(u8),
}

pub fn dehex_fixed_size<'a>(val: &'a str, into: &mut [u8]) -> Result<&'a str, DehexError> {
    fn nibble_from_char(ch: u8) -> Result<u8, DehexError> {
        match ch {
            b'A'...b'F' => Ok(ch - b'A' + 10),
            b'a'...b'f' => Ok(ch - b'a' + 10),
            b'0'...b'9' => Ok(ch - b'0'),
            _ => return Err(DehexError::InvalidChar(ch)),
        }
    }
    let mut consumed = 0;
    let mut inbytes = val.bytes();
    let mut outbytes = into.iter_mut();
    for oby in outbytes {
        let mut buf = 0;
        if let Some(ch) = inbytes.next() {
            buf |= nibble_from_char(ch)?;
        } else {
            return Err(DehexError::Truncated { odd: false });
        }
        buf <<= 4;
        if let Some(ch) = inbytes.next() {
            buf |= nibble_from_char(ch)?;
        } else {
            return Err(DehexError::Truncated { odd: true });
        }
        *oby = buf;
        consumed += 2;
    }
    Ok(&val[consumed..])
}

pub fn hex(val: &[u8]) -> Result<String, DehexError> {
    use std::io::Write;

    let mut out = Vec::new();
    for by in val.iter() {
        write!(&mut out, "{:02x}", by).unwrap();
    }
    Ok(String::from_utf8(out).unwrap())
}

pub fn dehex(val: &str) -> Result<Vec<u8>, DehexError> {
    let mut modulus = 0;
    let mut buf = 0;
    let mut output = Vec::with_capacity(val.len() / 2);

    for byte in val.bytes() {
        buf <<= 4;
        match byte {
            b'A'...b'F' => buf |= byte - b'A' + 10,
            b'a'...b'f' => buf |= byte - b'a' + 10,
            b'0'...b'9' => buf |= byte - b'0',
            _ => return Err(DehexError::InvalidChar(byte)),
        }

        modulus += 1;
        if modulus == 2 {
            modulus = 0;
            output.push(buf);
        }
    }

    Ok(output)
}