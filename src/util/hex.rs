
#[derive(Debug)]
pub enum DehexError {
    Truncated,
    TooLong,
    InvalidChar,
}

pub fn dehex_fixed_size(val: &str, into: &mut [u8]) -> Result<(), DehexError> {
    let mut modulus = 0;
    let mut buf = 0;
    let mut output = into.iter_mut();

    for byte in val.bytes() {
        buf <<= 4;
        match byte {
            b'A'...b'F' => buf |= byte - b'A' + 10,
            b'a'...b'f' => buf |= byte - b'a' + 10,
            b'0'...b'9' => buf |= byte - b'0',
            _ => return Err(DehexError::InvalidChar),
        }

        modulus += 1;
        if modulus == 2 {
            modulus = 0;
            match output.next() {
                Some(bslot) => *bslot = buf,
                None => return Err(DehexError::TooLong),
            }
        }
    }

    match output.next().is_some() {
        false => Ok(()),
        true => Err(DehexError::Truncated),
    }
}