use rust_template::add;

#[test]
fn err() {
    assert!(add(1, 2).is_err());
}

#[test]
fn ok() {
    assert_eq!(add(1, 1).unwrap(), 2);
}
