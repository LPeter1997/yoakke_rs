
pub fn assert_iter_eq<I1, I2>(mut i1: I1, mut i2: I2)
    where I1 : Iterator, I2 : Iterator,
    <I1 as Iterator>::Item : PartialEq<<I2 as Iterator>::Item> + std::fmt::Debug,
    <I2 as Iterator>::Item : std::fmt::Debug {
    loop {
        match (i1.next(), i2.next()) {
            (Some(a), Some(b)) => assert_eq!(a, b),
            (None, None) => return,
            (Some(a), None) => panic!("RHS terminates early ({:?})!", a),
            (None, Some(b)) => panic!("LHS terminates early ({:?})!", b),
        }
    }
}
