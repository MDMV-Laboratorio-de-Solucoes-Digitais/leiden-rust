//! `NodeId` trait for user-supplied graph node identifiers.

use std::hash::Hash;

/// A stable user-supplied identifier for a graph node.
///
/// The library does not assume any particular representation; callers may use
/// strings, integers, UUIDs, or any other `Hash + Eq` type. Internally, every
/// node is mapped to a dense `u32` index; that mapping is private and
/// preserved across all operations on a graph.
///
/// `Ord` is required so `RunResult::partition` can sort assignments by
/// user-supplied id (FR-001; `library-api.md §7`).
pub trait NodeId: Hash + Eq + Clone + Ord + std::fmt::Debug + 'static {}

impl<T> NodeId for T where T: Hash + Eq + Clone + Ord + std::fmt::Debug + 'static {}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher as _};

    use super::NodeId;

    fn assert_node_id<T: NodeId>() {}

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn string_implements_node_id() {
        assert_node_id::<String>();
        let first = String::from("alpha");
        let second = String::from("alpha");
        let third = String::from("beta");
        assert_eq!(first, second);
        assert!(first < third);
        assert_eq!(hash_of(&first), hash_of(&second));
        assert_ne!(hash_of(&first), hash_of(&third));
        let cloned = first.clone();
        assert_eq!(first, cloned);
    }

    #[test]
    fn u32_implements_node_id() {
        assert_node_id::<u32>();
        let first: u32 = 42;
        let second: u32 = 42;
        let third: u32 = 99;
        assert_eq!(first, second);
        assert!(first < third);
        assert_eq!(hash_of(&first), hash_of(&second));
        assert_ne!(hash_of(&first), hash_of(&third));
        let cloned = first;
        assert_eq!(first, cloned);
    }

    #[test]
    fn tuple_string_u32_implements_node_id() {
        assert_node_id::<(String, u32)>();
        let first = (String::from("n"), 1_u32);
        let second = (String::from("n"), 1_u32);
        let third = (String::from("n"), 2_u32);
        assert_eq!(first, second);
        assert!(first < third);
        assert_eq!(hash_of(&first), hash_of(&second));
        let cloned = first.clone();
        assert_eq!(first, cloned);
    }

    #[test]
    fn blanket_impl_covers_clone_ord_hash_eq() {
        #[expect(
            clippy::needless_pass_by_value,
            reason = "test helper intentionally takes values to exercise NodeId Clone"
        )]
        fn requires_all<T>(first: T, second: T)
        where
            T: NodeId + Ord,
        {
            let ord = first.cmp(&second);
            let eq = first == second;
            let cloned = first.clone();
            let hash = hash_of(&first);
            let _ = (ord, eq, cloned, hash);
        }
        requires_all(String::from("x"), String::from("y"));
        requires_all(1_u32, 2_u32);
    }
}
