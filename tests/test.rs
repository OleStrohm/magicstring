#![no_std]
use magicstring::MagicString;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_std_test() {
        let _ = MagicString::new([].as_slice());
    }
}
