use super::MagicString;

pub trait Sealed {}

impl<'a> Sealed for MagicString<'a> {}

