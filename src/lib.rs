use fnv::{FnvHashMap, FnvHasher};
use std::{
    collections::{HashMap, hash_map::Iter as HashMapIter},
    hash::BuildHasherDefault,
    slice::Iter,
};

/// `ExpirationList` is more performant than a `HashMap` for items that are likely to be removed
/// over time and require a stable ID which can be a `usize`. It does not automatically remove
/// items or track expiry. It should not be used when the expiration is fixed, in this case
/// there are other more efficient datastructures such as priority queues.
///
/// Example usage:
/// ```
/// use expiration_list::ExpirationList;
/// # use std::error::Error;
/// # fn main() -> Result<(), i32> {
/// 
/// let mut list = ExpirationList::new();
/// let value: i32 = 1234;
/// let id = list.add(value);
/// assert_eq!(list.get(id), Some(&value));
/// assert_eq!(list.contains(id), true);
/// 
/// let removed_value: i32 = list.remove(id).ok_or(0)?;
/// assert_eq!(removed_value, value);
/// assert_eq!(list.get(id), None);
/// assert_eq!(list.contains(id), false);
/// 
/// # Ok(())
/// # }
/// ```
/// 
/// `ExpirationList` stores new items in a `Vec<Option<T>>`. Removing an item sets it to None.
/// When more than half of the items in the `Vec` are removed, the `Vec` is shrunk in half.
/// Any items in the first half that are not yet removed are moved to a `HashMap<usize, T>`.
///
/// This has a number of advantages over using a `HashMap`:
///     - Direct access requires less hashing
///     - Less memory usage overall since the key is not stored for all elements
///     - Adding to a `Vec` is faster (no hashing, no collisions)
///
/// It also has the following advantages over using a plain `Vec`:
///     - Removes are O(1) + allows for a stable id for each element without requiring unlimited
///       memory
///     - Automatically shrinks
///
/// The more likely it is that old items are removed before new ones, the more performant
/// `ExpirationList` becomes due to less `HashMap` usage. In the best case, `ExpirationList`
/// performs like a `Vec` with some extra branching and O(1) removal time. In the worse case,
/// `ExpirationList` performs like a `HashMap` with some extra branching.
#[derive(Debug)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ExpirationList<T> {
    first_id: usize,
    count: usize,
    list: Vec<Option<T>>,
    map: HashMap<usize, T, BuildHasherDefault<FnvHasher>>,
}

pub struct ExpirationListIterator<'a, T> {
    map_iter: Option<HashMapIter<'a, usize, T>>,
    list_iter: Iter<'a, Option<T>>,
    list_id: usize,
}

impl<'a, T> Iterator for ExpirationListIterator<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.map_iter {
                Some(map_iter) => {
                    if let Some((key, value)) = map_iter.next() {
                        return Some((*key, value));
                    } else {
                        self.map_iter = None; // Continue to list iter
                    }
                }
                None => {
                    let next = self.list_iter.next();
                    self.list_id += 1;
                    match next {
                        Some(Some(value)) => return Some((self.list_id - 1, value)),
                        Some(None) => (), // Continue to next item in the list
                        None => return None,
                    }
                }
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a ExpirationList<T> {
    type Item = (usize, &'a T);
    type IntoIter = ExpirationListIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ExpirationListIterator {
            list_iter: self.list.iter(),
            map_iter: Some(self.map.iter()),
            list_id: self.first_id,
        }
    }
}

impl<T> Default for ExpirationList<T> {
    fn default() -> Self {
        ExpirationList {
            first_id: 0,
            count: 0,
            list: Vec::new(),
            map: FnvHashMap::default(),
        }
    }
}

impl<T> ExpirationList<T> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a new item to the `ExpirationList` and returns its stable ID.
    pub fn add(&mut self, value: T) -> usize {
        self.list.push(Some(value));
        self.count += 1;
        return self.first_id + self.list.len() - 1;
    }

    /// Removes an item by ID and returns `Some(item: T)` when the item was found and `None` when
    /// the item was not found.
    pub fn remove(&mut self, id: usize) -> Option<T> {
        if id < self.first_id {
            return self.map.remove(&id);
        }

        let mut removed_value = None;
        std::mem::swap(self.list.get_mut(id - self.first_id)?, &mut removed_value);
        if removed_value.is_none() {
            return None;
        }
        self.count -= 1;

        let original_len = self.list.len();
        if self.count * 2 < original_len && original_len > 32 {
            let mut shrink_count = self.list.len() / 2;
            let mut swap_list = Vec::with_capacity(original_len);
            std::mem::swap(&mut self.list, &mut swap_list);
            swap_list
                .into_iter()
                .enumerate()
                .for_each(|(idx, value): (usize, Option<T>)| {
                    if idx < shrink_count {
                        if let Some(inner_value) = value {
                            self.map.insert(idx + self.first_id, inner_value);
                            self.count -= 1;
                        }
                        // If the number of non-None items is still less than half the remaining length
                        // then keep shrinking.
                        let remaining_len = original_len - shrink_count;
                        if idx == shrink_count - 1
                            && self.count * 2 < remaining_len
                            && remaining_len > 32
                        {
                            shrink_count += remaining_len / 2;
                        }
                    } else {
                        self.list.push(value);
                    }
                });
            self.first_id += shrink_count;
        }

        return removed_value;
    }

    /// Takes an item ID and returns `Some(item: &T)` when the item is found and `None` otherwise.
    pub fn get(&self, id: usize) -> Option<&T> {
        if id < self.first_id {
            return self.map.get(&id);
        }
        return self.list.get(id - self.first_id)?.as_ref();
    }

    /// Takes an item ID and returns `Some(item: &mut T)` when the item is found and `None`
    /// otherwise.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        if id < self.first_id {
            return self.map.get_mut(&id);
        }
        return self.list.get_mut(id - self.first_id)?.as_mut();
    }

    /// Returns `true` if an item with the given ID exists. Returns `false` otherwise.
    pub fn contains(&self, id: usize) -> bool {
        if id < self.first_id {
            return self.map.contains_key(&id);
        } else if let Some(Some(_)) = self.list.get(id) {
            return true;
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_counts_elements() {
        let mut list = ExpirationList::new();
        for idx in 0..1024 {
            list.add(idx);
        }
        assert_eq!(list.count, 1024);

        for idx in 0..513 {
            list.remove(idx);
        }
        assert_eq!(list.count, 1024 - 513);

        for idx in 0..260 {
            list.remove(idx + 600); // 513 - 599 should be not removed
        }
        assert_eq!(list.count, 1024 - 513 - 260 - (600 - 513));
        assert_eq!(list.map.len(), 600 - 513);

        // Check that the list has been reduced to 256 items
        assert_eq!(list.first_id, 512 + 256);
        assert_eq!(list.list.len(), 256);
    }

    #[test]
    fn it_adds_and_gets() {
        let mut list = ExpirationList::new();
        for idx in 0..1024 {
            list.add(idx);
        }
        for idx in 0..1024 {
            assert_eq!(list.get(idx), Some(&idx));
        }
    }

    #[test]
    fn it_gets_after_remove() {
        let mut list = ExpirationList::new();
        for idx in 0..1024 {
            list.add(idx);
        }
        for idx in 0..513 {
            assert_eq!(list.remove(idx), Some(idx as i32));
        }
        assert_eq!(list.get(100), None);
        assert_eq!(list.remove(100), None);
        assert_eq!(list.get(600), Some(&600));
        assert_eq!(list.remove(600), Some(600));
        assert_eq!(list.get(600), None);

        list.into_iter();
    }

    #[test]
    fn it_is_iterable() {
        let mut list = ExpirationList::new();
        for idx in 0..10_000 {
            list.add(idx as i32);
            list.remove(idx);
        }
        for idx in 0..10 {
            list.add(idx);
        }
        list.remove(10_004);
        list.remove(10_007);
        list.remove(10_008);

        let mut result: Vec<i32> = Vec::new();
        for (key, value) in &list {
            assert_eq!(key as i32 - 10_000, *value);
            result.push(*value);
        }

        assert_eq!(result, vec![0, 1, 2, 3, 5, 6, 9]);
    }

    #[test]
    fn it_removes_multiple_times() {
        let mut list = ExpirationList::new();
        for idx in 0..10 {
            list.add(idx);
        }
        assert_eq!(list.remove(1), Some(1));
        for _ in 0..10 {
            assert_eq!(list.remove(1), None);
        }
        for idx in 0..10 {
            list.add(idx);
        }
        assert_eq!(list.count, 19);
    }
}
