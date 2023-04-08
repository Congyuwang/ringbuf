use crate::{prelude::*, storage::Static, LocalRb};

#[test]
fn push() {
    let mut rb = LocalRb::<Static<i32, 2>>::default();

    assert_eq!(rb.push_overwrite(0), None);
    assert_eq!(rb.push_overwrite(1), None);
    assert_eq!(rb.push_overwrite(2), Some(0));

    assert_eq!(rb.try_pop(), Some(1));
    assert_eq!(rb.try_pop(), Some(2));
    assert_eq!(rb.try_pop(), None);
}

#[test]
fn push_iter() {
    let mut rb = LocalRb::<Static<i32, 2>>::default();
    rb.push_iter_overwrite([0, 1, 2, 3, 4, 5].into_iter());
    assert!(rb.pop_iter().eq([4, 5]));
}

#[test]
fn push_slice() {
    let mut rb = LocalRb::<Static<i32, 2>>::default();
    rb.push_slice_overwrite(&[0, 1, 2, 3, 4, 5]);
    assert!(rb.pop_iter().eq([4, 5]));
}
