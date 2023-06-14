use std::{hash::Hash, str::Split, fmt};

pub struct MaybeEmptyIter<Iter: Iterator>(Option<Iter>);

impl<T: Iterator> Iterator for MaybeEmptyIter<T> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(Iterator::next)
    }
}

pub trait TrieKey: Clone + fmt::Debug + Eq + Hash {
    fn key_rest(&self) -> Option<(&str, &str)>;
    fn iter_keys(&self) -> MaybeEmptyIter<Split<'_, char>>;
    fn key(&self) -> Option<&str> {
        self.iter_keys().next()
    }
    fn join(&self, other: &str) -> String;
}

impl TrieKey for String {
    fn key_rest(&self) -> Option<(&str, &str)> {
        if self.is_empty() {
            None
        }
        else if let Some(offset) = self.rfind(' ') {
            Some((&self[offset + 1..], &self[..offset]))
        }
        else {
            Some((self, ""))
        }
    }

    fn iter_keys(&self) -> MaybeEmptyIter<Split<'_, char>> {
        MaybeEmptyIter((!self.is_empty()).then_some(self.split(' ')))
    }

    fn join(&self, other: &str) -> String {
        if self.is_empty() {
            other.to_string()
        } else {
            format!("{} {}", other, self)
        }
    }
}

impl TrieKey for &String {
    fn key_rest(&self) -> Option<(&str, &str)> {
        if self.is_empty() {
            None
        }
        else if let Some(offset) = self.rfind(' ') {
            Some((&self[offset + 1..], &self[..offset]))
        }
        else {
            Some((self, ""))
        }
    }

    fn iter_keys(&self) -> MaybeEmptyIter<Split<'_, char>> {
        MaybeEmptyIter((!self.is_empty()).then_some(self.split(' ')))
    }

    fn join(&self, other: &str) -> String {
        if self.is_empty() {
            other.to_string()
        } else {
            format!("{} {}", other, self)
        }
    }
}

impl TrieKey for &str {
    fn key_rest(&self) -> Option<(&str, &str)> {
        if self.is_empty() {
            None
        }
        else if let Some(offset) = self.rfind(' ') {
            Some((&self[offset + 1..], &self[..offset]))
        }
        else {
            Some((self, ""))
        }
    }

    fn iter_keys(&self) -> MaybeEmptyIter<Split<'_, char>> {
        MaybeEmptyIter((!self.is_empty()).then_some(self.split(' ')))
    }

    fn join(&self, other: &str) -> String {
        if self.is_empty() {
            other.to_string()
        } else {
            format!("{} {}", other, self)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::trie_key::TrieKey;

    #[test]
    fn test_triekey() {
        assert_eq!("a b c".key_rest(), Some(("c", "a b")));
        assert_eq!("a b".key_rest(), Some(("b", "a")));
        assert_eq!("a".key_rest(), Some(("a", "")));
        assert_eq!("".key_rest(), None);

        assert_eq!("a b c".iter_keys().collect::<Vec<_>>(), vec!["a", "b", "c"]);
        assert_eq!("a".iter_keys().next(), Some("a"));
        assert_eq!("".iter_keys().next(), None);
    }
}
