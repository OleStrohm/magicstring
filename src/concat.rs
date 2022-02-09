use std::iter::Chain;

use crate::MagicStringTrait;

#[derive(Clone, Copy)]
pub struct Concat<L, R> {
    left: L,
    right: R,
}

impl<L, R> Concat<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<'a, L, R> MagicStringTrait<'a> for Concat<L, R>
where
    L: MagicStringTrait<'a>,
    R: MagicStringTrait<'a>,
{
    type Iter = Chain<L::Iter, R::Iter>;
    type Bytes = Chain<L::Bytes, R::Bytes>;
    type Chars = Chain<L::Chars, R::Chars>;
    type CharIndices = Chain<L::CharIndices, R::CharIndices>;

    fn len(&self) -> usize {
        self.left.len() + self.right.len()
    }

    fn iter(&self) -> Self::Iter {
        self.left.iter().chain(self.right.iter())
    }

    fn bytes(self) -> Self::Bytes {
        self.left.bytes().chain(self.right.bytes())
    }

    fn chars(&self) -> Self::Chars {
        self.left.chars().chain(self.right.chars())
    }

    // TODO: This doesn't work because the indices in the right side are not updated
    fn char_indices(&self) -> Self::CharIndices {
        self.left.char_indices().chain(self.right.char_indices())
    }

    fn is_empty(&self) -> bool {
        self.left.is_empty() && self.right.is_empty()
    }

    // TODO: This is weird each output has to be a Concat<L, R>
    fn split_at(&self, index: usize) -> (Self, Self) {
        if index < self.left.len() {
            let (left_of_left, right_of_left) = self.left.split_at(index);
            let (left_of_right, right_of_right) = self.right.split_at(0);
            (
                Concat::new(left_of_left, left_of_right),
                Concat::new(right_of_left, right_of_right),
            )
        } else {
            let (left_of_left, right_of_left) = self.left.split_at(0);
            let (left_of_right, right_of_right) = self.right.split_at(index - self.left.len());
            (
                Concat::new(right_of_left, left_of_right),
                Concat::new(left_of_left, right_of_right),
            )
        }
    }

    fn trim_start(&self) -> Self {
        let left = self.left.trim_start();
        let right = if left.is_empty() {
            self.right.trim_start()
        } else {
            self.right
        };
        Concat::new(left, right)
    }

    fn trim_end(&self) -> Self {
        let right = self.right.trim_end();
        let left = if right.is_empty() {
            self.left.trim_end()
        } else {
            self.left
        };
        Concat::new(left, right)
    }

    fn pop(&mut self) -> Option<char> {
        self.right.pop().or_else(|| self.left.pop())
    }
}

#[cfg(test)]
mod test {
    use crate::{MagicString, MagicStringTrait};

    // #[test]
    // fn remove_middle_basic() {
    //     let s = ["some", "silly", "text"];
    //     let s = MagicString::new(&s);
    //     let (_, right) = s.split_at(4);
    //     let (_, right) = right.split_at(5);
    //     let actual = right.chars().collect::<String>();
    //     let expected = "text".to_string();
    //     assert_eq!(actual, expected);
    // }

    #[test]
    fn remove_middle() {
        let s = ["some", "silly", "text"];
        let s = MagicString::new(&s);
        let (left, right) = s.split_at(9);
        let left_actual = left.chars().collect::<String>();
        let right_actual = right.chars().collect::<String>();
        assert_eq!(left_actual, "somesilly".to_string());
        assert_eq!(right_actual, "text".to_string());
        let (left, _) = left.split_at(4);
        let s = left.concat(right);
        let actual = s.chars().collect::<String>();
        let expected = "sometext".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn multi_concat() {
        let s1 = ["a", "b"];
        let s2 = ["c", "d"];
        let s3 = ["e", "f"];
        let s4 = ["g", "h"];
        let s1 = MagicString::new(&s1);
        let s2 = MagicString::new(&s2);
        let s3 = MagicString::new(&s3);
        let s4 = MagicString::new(&s4);
        let line = s1.concat(s2).concat(s3).concat(s4);
        let line_chars = line.chars();
        let line_actual = line_chars.collect::<String>();
        let tree = (s1.concat(s2)).concat(s3.concat(s4));
        let tree_chars = tree.chars();
        let tree_actual = tree_chars.collect::<String>();
        let expected = "abcdefgh".to_string();
        assert_eq!(line_actual, expected);
        assert_eq!(line_actual, tree_actual);
    }

    // TODO: This doesn't work
    // #[test]
    // fn char_indices() {
    //     let left = ["a", "🍅", "b"];
    //     let right = ["c", "d"];
    //     let left = MagicString::new(&left);
    //     let right = MagicString::new(&right);
    //     let string = left.concat(right);
    //     let mut chars = string.char_indices();
    //     assert_eq!(chars.next().unwrap(), (0, 'a'));
    //     assert_eq!(chars.next().unwrap(), (1, '🍅'));
    //     assert_eq!(chars.next().unwrap(), (5, 'b'));
    //     // TODO: The wrong starts here
    //     assert_eq!(chars.next().unwrap(), (6, 'c'));
    //     assert_eq!(chars.next().unwrap(), (7, 'd'));
    // }

    #[test]
    fn collect() {
        let left = ["a", "b"];
        let right = ["c", "d"];
        let left = MagicString::new(&left);
        let right = MagicString::new(&right);
        let string = left.concat(right);
        let chars = string.chars();
        let actual = chars.collect::<String>();
        let expected = "abcd".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn split() {
        let left = ["0123", "4", "56"];
        let right = ["c", "d"];
        let left = MagicString::new(&left);
        let right = MagicString::new(&right);
        let string = left.concat(right);
        let (left, right) = string.split_at(3);

        let actual = left.iter().collect::<String>();
        let expected = "012".to_string();
        assert_eq!(expected, actual);

        let actual = right.iter().collect::<String>();
        let expected = "3456cd".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn small_trims() {
        let left = ["  "];
        let left = MagicString::new(&left);
        let actual = left.trim_start().chars().collect::<String>();
        let expected = "".to_string();
        assert_eq!(expected, actual);
        let actual = left.trim_end().chars().collect::<String>();
        let expected = "".to_string();
        assert_eq!(expected, actual);

        let left = ["a"];
        let right = ["  "];
        let left = MagicString::new(&left);
        let right = MagicString::new(&right);
        let s = left.concat(right);
        let actual = s.trim_end().chars().collect::<String>();
        let expected = "a".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn big_trims() {
        let left = ["   ", " ", "  "];
        let mid = [" ", "T", " "];
        let right = [" ", "     ", " "];
        let left = MagicString::new(&left);
        let mid = MagicString::new(&mid);
        let right = MagicString::new(&right);
        let s = left.concat(mid).concat(right);
        let actual = s.trim_start().chars().collect::<String>();
        let expected = "T        ".to_string();
        assert_eq!(expected, actual);

        let actual = s.trim_end().chars().collect::<String>();
        let expected = "       T".to_string();
        assert_eq!(expected, actual);

        let actual = s.trim().chars().collect::<String>();
        let expected = "T".to_string();
        assert_eq!(expected, actual);
    }
}
