#![deny(missing_docs)]
//! A zero allocations string type made up of string slices.
//!
//! ```
//! use magicstring::{MagicString, Contains, Find};
//! let input = ["012", "34", "5"];
//! let string = MagicString::new(&input);
//!
//! assert!(string.contains('3'));
//! assert_eq!(string.find('3').unrwap(), 3);
//! ```
//!
//! To use [`MagicString::find`] and [`MagicString::contains`] 
//! import `magicstring::{Find, Contains}`.
use core::fmt;
use core::str::Bytes as StdBytes;
use core::str::CharIndices as StdCharIndices;
use core::str::Chars as StdChars;

use unicode_width::UnicodeWidthStr;

mod contains;
mod find;
mod fromrange;
mod sealed;

use fromrange::FromRange;

pub use contains::Contains;
pub use find::Find;

// -----------------------------------------------------------------------------
//     - Slice offset -
//     This represents the offset inside a slice
//     Start is the offset from the start
//
//     End is the offset from the end.
//     Given an offset of 1, the end is at slice.len() - end_offset
//
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
enum Offset {
    None,
    Start(usize),
    End(usize),
    StartEnd(usize, usize),
}

// -----------------------------------------------------------------------------
//     - Magic iterator -
//     It's not really magic
// -----------------------------------------------------------------------------
/// Iterator over the inner string slices
pub struct MagicIter<'a> {
    inner: &'a [&'a str],
    index: usize,
    offset: Offset,
}

impl<'a> MagicIter<'a> {
    fn next_by_index(&mut self, forward: bool) -> Option<&'a str> {
        if self.index == self.inner.len() {
            return None;
        }

        let index = match forward {
            true => self.index,
            false => self.inner.len() - 1 - self.index,
        };

        let len = self.inner[index].len();
        let (start, end) = match (self.offset, index) {
            (Offset::Start(start), 0) => (start, self.inner[0].len()),
            (Offset::StartEnd(start, end), 0) if self.inner.len() == 1 => (start, len - end),
            (Offset::StartEnd(start, _), 0) => (start, len),
            (Offset::End(end), index) if index == self.inner.len() - 1 => (0, len - end),
            (Offset::StartEnd(_, end), index) if index == self.inner.len() - 1 => (0, len - end),
            (_, _) => (0, len),
        };

        let ret = &self.inner[index][start..end];

        self.index += 1;

        Some(ret)
    }
}

impl<'a> Iterator for MagicIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_by_index(true)
    }
}

impl<'a> DoubleEndedIterator for MagicIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_by_index(false)
    }
}

// -----------------------------------------------------------------------------
//     - Magic string -
// -----------------------------------------------------------------------------
/// A zero allocations string made up of string slices.
/// Cheap to copy
#[derive(Copy, Clone)]
pub struct MagicString<'a> {
    inner: &'a [&'a str],
    offset: Offset,
}

impl<'a> MagicString<'a> {
    /// Create a new instance of a `MagicString` from string slices.
    pub fn new(inner: &'a [&'a str]) -> Self {
        Self { inner, offset: Offset::None }
    }

    /// The total length of the string in bytes
    pub fn len(&self) -> usize {
        self.iter().map(|s| s.len()).sum()
    }

    /// Returns true if this string has a length of zero, otherwise false
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// An iterator over the bytes of the inner string slices
    pub fn bytes(&self) -> Bytes {
        Bytes::new(self.iter())
    }

    /// An iterator over the characters of the inner string slices
    pub fn chars(&self) -> Chars<'a> {
        Chars::new(self.iter())
    }

    /// An iterator over the characters and their index (byte position) of the inner string slices
    pub fn char_indices(&self) -> CharIndices<'a> {
        CharIndices::new(self.iter())
    }

    /// Split the string in two:
    pub fn split_at(&self, index: usize) -> (Self, Self) {
        let (slice, index) = self.index(index);

        let left = &self.inner[..=slice];
        let right = &self.inner[slice..];

        let left_len = left[left.len() - 1].len();
        let left_offset = match self.offset {
            Offset::Start(start) => {
                // Silly bits
                let end = left_len - start - index;
                Offset::StartEnd(start, end)
            }
            Offset::StartEnd(start, end) => {
                let lark = index + start + end;
                Offset::StartEnd(start, end + left_len - lark)
            }
            _ => Offset::End(left_len - index),
        };

        let right_offset = match self.offset {
            Offset::Start(start) => Offset::Start(start + index),
            Offset::StartEnd(start, end) => Offset::StartEnd(start + index, end),
            Offset::End(end) => Offset::StartEnd(index, end),
            _ => Offset::Start(index),
        };

        (Self::from_split(left_offset, left), Self::from_split(right_offset, right))
    }

    /// Trim any white space from the start and the end of the string.
    /// Unlike [`&str`] this does not work for RTL.
    pub fn trim(&self) -> Self {
        self.trim_start().trim_end()
    }

    /// Trim the start of the string from any white space characters.
    /// This will not work correctly with RTL
    pub fn trim_start(&self) -> Self {
        let mut slice_index = 0;
        let mut char_index = 0;

        for (index, slice) in self.iter().enumerate() {
            let trimmed = slice.trim_start();
            if trimmed.is_empty() {
                continue;
            }
            slice_index = index;
            char_index = slice.len() - trimmed.len();
            break;
        }

        let offset = match self.offset {
            Offset::End(end) => Offset::StartEnd(char_index, end),
            Offset::Start(start) => Offset::Start(start + char_index),
            Offset::StartEnd(start, end) => Offset::StartEnd(start + char_index, end),
            _ => Offset::Start(char_index),
        };

        Self::from_split(offset, &self.inner[slice_index..])
    }

    /// Trim the end of the string from any white space characters.
    /// This will not work correctly with RTL
    pub fn trim_end(&self) -> Self {
        let mut slice_index = self.inner.len();
        let mut char_index = 0;
        let mut slice_len = 0;

        for (i, slice) in self.iter().rev().enumerate() {
            slice_index = self.inner.len() - i;
            let trimmed = slice.trim_end();
            if trimmed.is_empty() {
                continue;
            }
            slice_len = slice.len();
            char_index = trimmed.len();
            break;
        }

        let offset = match self.offset {
            Offset::Start(start) => Offset::StartEnd(start, slice_len - char_index),
            Offset::End(end) => Offset::End(end + slice_len - char_index),
            Offset::StartEnd(start, end) => Offset::StartEnd(start, end + slice_len - char_index),
            Offset::None => Offset::End(slice_len - char_index),
        };

        Self::from_split(offset, &self.inner[..slice_index])
    }

    /// Get a [`MagicString`] from a range.
    /// ```
    /// use magicstring::MagicString;
    /// let input = ["012", "345"];
    /// let string = MagicString::new(&input);
    /// // Exclusive range
    /// let value = string.get(1..5);
    /// assert_eq!(value.to_string(), "1234".to_string());
    ///
    /// // Inclusive range
    /// let value = string.get(1..=5);
    /// assert_eq!(value.to_string(), "12345".to_string());
    ///
    /// // Range to
    /// let value = string.get(..5);
    /// assert_eq!(value.to_string(), "01234".to_string());
    /// ```
    pub fn get(&self, range: impl FromRange) -> Self {
        let (start, end) = range.into_start_end(self.len());

        let (ret, _) = self.split_at(end);
        let (_, ret) = ret.split_at(start);
        ret
    }

    /// Produce an iterator over the inner string slices.
    pub fn iter(&self) -> MagicIter<'a> {
        MagicIter { inner: self.inner, offset: self.offset, index: 0 }
    }

    /// Remove the last char from the string
    /// ```
    /// use magicstring::MagicString;
    /// let input = ["012", "345"];
    /// let mut string = MagicString::new(&input);
    /// let c = string.pop().unwrap();
    /// assert_eq!(c, '5');
    /// assert_eq!(string.to_string(), "01234".to_string());
    /// ```
    pub fn pop(&mut self) -> Option<char> {
        let c = self.iter().rev().next().and_then(|s| s.chars().last())?;

        let remove = c.len_utf8();
        let to = self.len() - remove;
        *self = self.get(..to);

        Some(c)
    }

    // Index is operating on processed slices
    fn index(&self, index: usize) -> (usize, usize) {
        let mut offset = 0;
        for (slice_index, slice) in self.iter().enumerate() {
            if index - offset > slice.len() {
                offset += slice.len();
                continue;
            }

            return (slice_index, index - offset);
        }

        panic!("index out of range");
    }

    fn from_split(offset: Offset, inner: &'a [&'a str]) -> Self {
        Self { inner, offset }
    }
}

// -----------------------------------------------------------------------------
//     - Display -
// -----------------------------------------------------------------------------
impl<'a> fmt::Display for MagicString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for slice in self.iter() {
            write!(f, "{slice}")?;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//     - Debug -
// -----------------------------------------------------------------------------
impl<'a> fmt::Debug for MagicString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for slice in self.iter() {
            write!(f, "{slice:?}")?;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//     - Unicode width -
// -----------------------------------------------------------------------------
impl<'a> UnicodeWidthStr for MagicString<'a> {
    fn width(&self) -> usize {
        self.iter().map(|s| s.width()).sum()
    }

    fn width_cjk(&self) -> usize {
        self.iter().map(|s| s.width_cjk()).sum()
    }
}

// -----------------------------------------------------------------------------
//     - Bytes -
// -----------------------------------------------------------------------------
/// An iterator over the bytes of the [`MagicString`]
pub struct Bytes<'a> {
    inner: MagicIter<'a>,
    current: Option<StdBytes<'a>>,
}

impl<'a> Bytes<'a> {
    fn new(mut inner: MagicIter<'a>) -> Self {
        let current = inner.next().map(|s| s.bytes());
        Self { inner, current }
    }
}

impl<'a> Iterator for Bytes<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.as_mut()?.next() {
            Some(s) => Some(s),
            None => {
                let slice = self.inner.next()?;
                self.current = Some(slice.bytes());
                self.current.as_mut()?.next()
            }
        }
    }
}

// -----------------------------------------------------------------------------
//     - Chars -
// -----------------------------------------------------------------------------
/// An iterator over the chars of the [`MagicString`]
pub struct Chars<'a> {
    inner: MagicIter<'a>,
    current: Option<StdChars<'a>>,
}

impl<'a> Chars<'a> {
    fn new(mut inner: MagicIter<'a>) -> Self {
        let current = inner.next().map(|s| s.chars());
        Self { inner, current }
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.as_mut()?.next() {
            Some(s) => Some(s),
            None => {
                let slice = self.inner.next()?;
                self.current = Some(slice.chars());
                self.current.as_mut()?.next()
            }
        }
    }
}

// -----------------------------------------------------------------------------
//     - Char indices -
// -----------------------------------------------------------------------------
/// An iterator over the characters and their index of the [`MagicString`]
pub struct CharIndices<'a> {
    inner: MagicIter<'a>,
    current: Option<(usize, StdCharIndices<'a>)>,
    offset: usize,
}

impl<'a> CharIndices<'a> {
    fn new(mut inner: MagicIter<'a>) -> Self {
        let current = inner.next().map(|s| (s.len(), s.char_indices()));
        Self { inner, current, offset: 0 }
    }
}

impl<'a> Iterator for CharIndices<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let (len, indices) = self.current.as_mut()?;

        match indices.next() {
            Some((i, c)) => Some((i + self.offset, c)),
            None => {
                self.offset += *len;
                self.current = self.inner.next().map(|s| (s.len(), s.char_indices()));
                self.next()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytes() {
        let s = ["a", "b"];
        let string = MagicString::new(&s);
        let mut bytes = string.bytes();
        assert_eq!(bytes.next().unwrap(), b'a');
        assert_eq!(bytes.next().unwrap(), b'b');
    }

    #[test]
    fn chars() {
        let s = ["a", "b"];
        let string = MagicString::new(&s);
        let mut chars = string.chars();
        assert_eq!(chars.next().unwrap(), 'a');
        assert_eq!(chars.next().unwrap(), 'b');
    }

    #[test]
    fn char_indices() {
        let s = ["a", "🍅", "b"];
        let string = MagicString::new(&s);
        let mut chars = string.char_indices();
        assert_eq!(chars.next().unwrap(), (0, 'a'));
        assert_eq!(chars.next().unwrap(), (1, '🍅'));
        assert_eq!(chars.next().unwrap(), (5, 'b'));
    }

    #[test]
    fn collect() {
        let s = ["a", "b"];
        let string = MagicString::new(&s);
        let chars = string.chars();
        let actual = chars.collect::<String>();
        let expected = "ab".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn split() {
        let s = ["0123", "4", "56"];
        let string = MagicString::new(&s);
        let (left, right) = string.split_at(3);

        let actual = left.iter().collect::<String>();
        let expected = "012".to_string();
        assert_eq!(expected, actual);

        let actual = right.iter().collect::<String>();
        let expected = "3456".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn split_left() {
        let s = ["0"];
        let string = MagicString::new(&s);
        let (left, _) = string.split_at(1);
        assert_eq!(left.iter().collect::<String>(), "0".to_string());
    }

    #[test]
    fn split_right() {
        let s = ["0"];
        let string = MagicString::new(&s);
        let (_, right) = string.split_at(0);
        assert_eq!(right.iter().collect::<String>(), "0".to_string());
    }

    #[test]
    fn trim_start() {
        let s = ["   ", "  ", "a"];
        let string = MagicString::new(&s);
        let actual = format!("{}", string.trim_start());
        let expected = "a".to_string();
        assert_eq!(expected, actual);

        let actual = format!("{}", string.trim_start().trim_end());
        let expected = "a".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn trim_end() {
        let s = ["a    ", " ", "  "];
        let string = MagicString::new(&s);
        let actual = format!("{}", string.trim_end());
        let expected = "a".to_string();
        assert_eq!(expected, actual);

        let actual = format!("{}", string.trim_end().trim_start());
        let expected = "a".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn trim() {
        let s = ["  ", " ", "  a    ", " ", "  "];
        let string = MagicString::new(&s);
        let actual = format!("{}", string.trim());
        let expected = "a".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn split_twice() {
        let s = ["012345"];
        let string = MagicString::new(&s);
        let (_, right) = string.split_at(3);
        let (left, _) = right.split_at(2);
        let actual = format!("{}", left);
        let expected = "34".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn iter_backwards() {
        let s = ["0", "1", "2"];
        let string = MagicString::new(&s);

        let mut iter = string.iter();

        assert_eq!(iter.next_back().unwrap(), "2");
        assert_eq!(iter.next_back().unwrap(), "1");
        assert_eq!(iter.next_back().unwrap(), "0");
    }

    #[test]
    fn len() {
        let s = ["12", "3"];
        let string = MagicString::new(&s);
        assert_eq!(string.len(), 3);

        let s = [""];
        let string = MagicString::new(&s);
        assert_eq!(string.len(), 0);

        let s = [];
        let string = MagicString::new(&s);
        assert_eq!(string.len(), 0);
    }

    #[test]
    fn pop() {
        let s = ["0", "1", "2"];
        let mut string = MagicString::new(&s);
        let actual = string.pop().unwrap();
        let expected = '2';
        assert_eq!(expected, actual);
    }
}
