use super::MagicString;


/// Finds the position of either a [`char`] or a slice of chars.
pub trait Find<P> : crate::sealed::Sealed {
    /// Find the pattern inside the string, starting from the beginning of the string
    fn find(&self, pat: P) -> Option<usize>;
    /// Find the pattern inside the string, starting from the end of the string
    fn rfind(&self, pat: P) -> Option<usize>;
}

impl<'a> Find<char> for MagicString<'a> {
    fn find(&self, pat: char) -> Option<usize> {
        let mut offset = 0;
        for s in self.iter() {
            match s.find(pat) {
                Some(pos) => return Some(pos + offset),
                None => offset += s.len(),
            }
        }

        None
    }

    fn rfind(&self, pat: char) -> Option<usize> {
        let mut offset = self.len();
        for s in self.iter().rev() {
            offset -= s.len();
            match s.rfind(pat) {
                Some(pos) => return Some(pos + offset),
                None => continue,
            }
        }

        None
    }
}

impl<'a> Find<&[char]> for MagicString<'a> {
    fn find(&self, pat: &[char]) -> Option<usize> {
        let mut offset = 0;
        for s in self.iter() {
            match s.find(pat) {
                Some(pos) => return Some(pos + offset),
                None => offset += s.len(),
            }
        }

        None
    }

    fn rfind(&self, pat: &[char]) -> Option<usize> {
        let mut offset = self.len();
        for s in self.iter().rev() {
            offset -= s.len();
            match s.rfind(pat) {
                Some(pos) => return Some(pos + offset),
                None => continue,
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn find_by_char() {
        let input = ["ab ", "c e", "fg"];
        let string = MagicString::new(input.as_slice());

        // Find first space
        let actual = string.find(' ').unwrap();
        let expected = 2;
        assert_eq!(expected, actual);

        // Find 'e'
        let actual = string.find('e').unwrap();
        let expected = 5;
        assert_eq!(expected, actual);
    }

    #[test]
    fn find_by_slice() {
        let input = ["ab ", "c e", "fg"];
        let string = MagicString::new(input.as_slice());

        // Find first space
        let actual = string.find(['|', ' '].as_slice()).unwrap();
        let expected = 2;
        assert_eq!(expected, actual);

        // Find first space
        let actual = string.find(['f', 'g', '%'].as_slice()).unwrap();
        let expected = 6;
        assert_eq!(expected, actual);
    }

    #[test]
    fn rfind_by_char() {
        let s = ["12", "3$45", "6$7", "89"];
        let string = MagicString::new(&s);
        let pos = string.rfind('$').unwrap();
        let string = string.get(..=pos);
        let actual = format!("{string}");
        let expected = String::from("123$456$");
        assert_eq!(expected, actual);
    }

    #[test]
    fn rfind_by_slice() {
        let s = ["123$456$789"];
        let string = MagicString::new(&s);
        let pos = string.rfind(['|', '$'].as_slice()).unwrap();
        let string = string.get(..=pos);
        let actual = format!("{string}");
        let expected = String::from("123$456$");
        assert_eq!(expected, actual);
    }

    #[test]
    fn rfind_on_substring() {
        let s = ["01", "23", "4567"];
        let string = MagicString::new(&s);
        let substring = string.get(1..5);
        let pos = substring.rfind('3').unwrap();

        let substring = substring.get(..pos);
        let actual = format!("{substring}");
        let expected = String::from("123");
        assert_eq!(expected, actual);
    }
}
