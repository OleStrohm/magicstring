use super::MagicString;

pub trait Contains<P> {
    fn contains(&self, pat: P) -> bool;
}

impl<'a> Contains<char> for MagicString<'a> {
    fn contains(&self, pat: char) -> bool {
        self.iter().any(|s| s.contains(pat))
    }
}

impl<'a> Contains<&[char]> for MagicString<'a> {
    fn contains(&self, pat: &[char]) -> bool {
        self.iter().any(|s| s.contains(pat))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contains_char() {
        let s = ["ab", "cd"];
        let string = MagicString::new(&s);
        assert!(string.contains('a'));
        assert!(string.contains('b'));
        assert!(string.contains('c'));
        assert!(string.contains('d'));

        assert!(!string.contains('e'));

        let (left, _) = string.split_at(3);
        assert!(left.contains('c'));
    }

    #[test]
    fn contains_slice_o_chars() {
        let s = ["ab", "cd"];
        let string = MagicString::new(&s);
        assert!(string.contains(['a', 'x'].as_slice()));
        assert!(!string.contains(['y', 'x'].as_slice()));
    }
}
