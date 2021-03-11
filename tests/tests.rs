use fakemap::FakeMap;
use serde_yaml::from_str;

fn map() -> FakeMap<i32, i32> {
    from_str(
        r#"---
1: 1
2: 4
3: 9
4: 16
5: 25
6: 36
"#,
    )
    .unwrap()
}

#[test]
fn remove() {
    let mut map = map();
    assert_eq!(Some(16), map.remove(&4));
    assert_eq!(None, map.remove(&4));
    assert_eq!(Some(9), map.remove(&3));
}

#[test]
fn add_remove() {
    let mut map = map();
    map.insert(10, 100);
    assert_eq!(Some(100), map.remove(&10));
    assert_eq!(None, map.remove(&10));
    map.insert(10, 77);
    map.insert(10, 99);

    assert_eq!(Some(&99), map.get(&10));
    assert_eq!(Some(99), map.remove(&10));
    assert_eq!(None, map.remove(&10));
}

#[test]
fn correct_order() {
    let mut map = map();
    map.insert(42, 42);
    map.insert(10, 100);
    assert_eq!(Some(100), map.remove(&10));
    assert_eq!(None, map.remove(&10));
    map.insert(10, 77);
    map.insert(10, 99);

    assert_eq!(Some(&99), map.get(&10));
    assert_eq!(Some(99), map.remove(&10));
    assert_eq!(None, map.remove(&10));

    map.remove(&2);
    map.insert(2, 5);

    assert_eq!(from_str::<FakeMap<i32, i32>>(r#"---
1: 1
3: 9
4: 16
5: 25
6: 36
42: 42
2: 5
"#).unwrap(), map);
}
