/**
 * A trait to match primitives with the parser.
 */

pub trait Parser {
    type Item;
}

pub trait Match<T> where Self : Parser {
    fn matches(a: &<Self as Parser>::Item, b: &T) -> bool;
    fn show_expected(item: &T) -> String;
}

pub trait ShowFound where Self : Parser {
    fn show_found(item: &<Self as Parser>::Item) -> String;
}
