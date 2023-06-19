use std::{collections::HashMap, fmt};

use crate::TrieKey;

#[derive(Default)]
pub struct Trie<V> {
    root: TrieNode<V>,
}

impl<V: Clone> Clone for Trie<V> {
    fn clone(&self) -> Self {
        Trie { root: self.root.clone() }
    }
}

impl<V: fmt::Debug> fmt::Debug for Trie<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Trie");
        builder.field("root", &self.root);
        builder.finish()
    }
}

#[allow(dead_code)]
impl<V> Trie<V> {
    pub fn new() -> Self {
        Self { root: TrieNode::default() }
    }

    pub fn lookup(&self, key: &str) -> Option<&V> {
        let mut current = &self.root;
        for part in key.iter_keys() {
            current = current.children.get(part)?;
        }
        current.value.as_ref()
    }

    pub fn children(&self, key: &str) -> Option<impl Iterator<Item=(&str, &V)>> {
        let mut current = &self.root;
        for part in key.iter_keys() {
            current = current.children.get(part)?;
        }
        Some(current.children.iter().filter_map(|(k, v)| Some((k.as_str(), v.value.as_ref()?))))
    }

    pub fn child_values(&self, key: &str) -> Option<impl Iterator<Item=&V>> {
        let mut current = &self.root;
        for part in key.iter_keys() {
            current = current.children.get(part)?;
        }
        Some(current.children.values().filter_map(|v| v.value.as_ref()))
    }

    pub fn child_keys(&self, key: &str) -> Option<impl Iterator<Item=&str>> {
        let mut current = &self.root;
        for part in key.iter_keys() {
            current = current.children.get(part)?;
        }
        Some(current.children.keys().map(String::as_str))
    }

    pub fn root_value(&self) -> Option<&V> {
        self.root.value.as_ref()
    }

    // pub fn root_child_keys(&self) -> Option<impl Iterator<Item=&str>> {
    //     Some(self.root.children.keys().map(String::as_str))
    // }

    pub fn insert(&mut self, key: impl TrieKey, value: V) -> Option<V> {
        let mut current = &mut self.root;
        for key in key.iter_keys() {
            current = current.children.entry(key.to_string()).or_insert_with(TrieNode::default); // TODO
        }
        current.value.replace(value)
    }

    pub fn delete(&mut self, key: &str) {
        Self::delete_recurse(&mut self.root, key);
    }

    fn delete_recurse(node: &mut TrieNode<V>, key: &str) -> bool {
        if key.is_empty() {
            node.value = None;
            return node.children.is_empty();
        }

        if let Some((key, rest)) = key.key_rest() {
            if Self::delete_recurse(node.children.get_mut(key).unwrap(), rest) {
                node.children.remove(key);
            }
        }

        false
    }

    pub fn aggregate_depth_first_root<Agg, F>(&mut self, visitor: &mut F) -> Agg
    where
        F: Fn(Option<&mut V>, String, Vec<Agg>) -> Agg
    {
        let result = self.aggregate_depth_first(visitor);
        visitor(self.root.value.as_mut(), String::new(), result)
    }

    pub fn aggregate_depth_first<Agg, F: VisitingAggregator<V, Agg>>(&mut self, visitor: &mut F) -> Vec<Agg>
    {
        Self::visit_depth_recurse(&mut self.root, "", visitor)
    }

    fn visit_depth_recurse<Agg, F: VisitingAggregator<V, Agg>>(node: &mut TrieNode<V>, path: &str, visitor: &mut F) -> Vec<Agg>
    {
        let mut results = vec![];
        for (key, child) in node.children.iter_mut() {
            let new_key = path.join(key);
            let result = Self::visit_depth_recurse(child, &new_key, visitor);
            results.push(visitor.visit(child.value.as_mut(), new_key, result));
        }
        results
    }
}

impl<V> Extend<(String, V)> for Trie<V> {
    fn extend<T: IntoIterator<Item=(String, V)>>(&mut self, iter: T) {
        for item in iter {
            self.insert(item.0, item.1);
        }
    }
}

pub struct TrieNode<V> {
    pub(crate) children: HashMap<String, TrieNode<V>>,
    pub(crate) value: Option<V>,
}

impl<V> Default for TrieNode<V> {
    fn default() -> Self {
        TrieNode {
            children: HashMap::new(),
            value: None,
        }
    }
}

impl<V: Clone> Clone for TrieNode<V> {
    fn clone(&self) -> Self {
        TrieNode {
            children: self.children.clone(),
            value: self.value.clone(),
        }
    }
}

impl<V: fmt::Debug> fmt::Debug for TrieNode<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("TrieNode");
        builder.field("value", &self.value);
        builder.field("children", &self.children);
        builder.finish()
    }
}

pub trait VisitingAggregator<V, Agg> {
    fn visit(&mut self, value: Option<&mut V>, key: String, memo: Vec<Agg>) -> Agg;
}

impl<F, V, Agg> VisitingAggregator<V, Agg> for F
where
    F: Fn(Option<&mut V>, String, Vec<Agg>) -> Agg
{
    fn visit(&mut self, value: Option<&mut V>, key: String, memo: Vec<Agg>) -> Agg {
        self(value, key, memo)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::Trie;
    #[test]
    fn test_insert() {
        let mut trie = Trie::new();
        trie.insert("auth", "auth");
        trie.insert("get device bundles", "get device bundles");
        trie.insert("get device", "get device");

        assert_eq!(trie.child_keys("").map(Iterator::collect), Some(HashSet::from(["auth", "get"])));
        assert_eq!(trie.lookup("auth"), Some(&"auth"));
        assert_eq!(trie.lookup("get device bundles"), Some(&"get device bundles"));
        assert_eq!(trie.lookup("get device"), Some(&"get device"));
    }
}
