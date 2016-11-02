use errors::UnhexlifyError;


// Returns the binary data represented by the hexadecimal string 'hexstr'. 'hexstr'
// must contain an even number of hexadecimal digits (which can be upper or lower case),
// otherwise an Error exception is raised.
//
// The Mikrotik Python API example makes use of binascii.unhexlify to parse the challenge
// returned by the router, so this is an copy/implementation of it.
//
// Mikrotik Python3 example:
//     http://wiki.mikrotik.com/wiki/Manual:API_Python3
//
// Python 3 binascii.unhexlify
//     https://docs.python.org/3/library/binascii.html#binascii.unhexlify
//
pub fn unhexlify<'a>(hexstr: &str) -> Result<Vec<u8>, UnhexlifyError> {
    let string_len = hexstr.len();
    if string_len == 0 || string_len % 2 != 0 {
        return Err(UnhexlifyError::OddCharacterCount(string_len));
    }

    let mut result: Vec<u8> = Vec::with_capacity(string_len / 2);
    let mut i = 0;
    let chars: Vec<char> = hexstr.chars().collect();

    while i < chars.len() - 1 {
        let mut c = chars[i];
        let top = try!(c.to_digit(16).ok_or(UnhexlifyError::InvalidHexDigit(c))) as u8;

        c = chars[i + 1];
        let bottom = try!(c.to_digit(16).ok_or(UnhexlifyError::InvalidHexDigit(c))) as u8;

        let parsed_num = (top << 4) + bottom;
        result.push(parsed_num);

        i += 2;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use errors::UnhexlifyError;

    #[test]
    fn test_valid_hex_string() {
        let hexstr = String::from("F0AB");
        let result = unhexlify(&hexstr).expect("should return a valid Vec<u8>");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 240);
        assert_eq!(result[1], 171);
    }

    #[test]
    fn test_invalid_hex_strings() {
        let invalid_strings = vec!["", "0", "123"];

        for s in invalid_strings {
            let invalid_string = String::from(s);
            let error = unhexlify(&invalid_string).unwrap_err();

            assert_eq!(error,
                       UnhexlifyError::OddCharacterCount(invalid_string.len()));
        }
    }

    #[test]
    fn test_invalid_hex_digits() {
        let invalid_strings = vec!["0Z", "Z0"];

        for invalid_string in invalid_strings {
            let error = unhexlify(&String::from(invalid_string)).unwrap_err();

            assert_eq!(error, UnhexlifyError::InvalidHexDigit('Z'));
        }
    }
}
