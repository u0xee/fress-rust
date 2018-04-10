//! A cohesive fressian library for rust

mod bit;
mod dispatch;
mod keyword;
mod list;
mod map;
mod memory;
mod method_union;
mod rational;
mod set;
mod sorted_map;
mod sorted_set;
mod string;
mod symbol;
mod value;
mod vector;

pub use value::Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testbed() {
        let x = Value { handle: 7 };

    }

    #[test]
    fn is_immediate() {
        assert!(Value {handle: 7}.is_immediate())
    }

    #[test]
    fn is_not() {
        assert!(Value::NIL.is_not() && Value::FALSE.is_not())
    }

    #[test]
    fn is_so() {
        assert!(Value {handle: 0}.is_so())
    }

    #[test]
    fn is_nil() {
        assert!(Value {handle: 7}.is_nil())
    }

    #[test]
    fn is_true() {
        assert!(Value {handle: !0}.is_true())
    }

    #[test]
    fn is_false() {
        assert!(Value {handle: !0 - 8}.is_false())
    }

    #[test]
    fn is_immediate_number() {
        assert!(Value {handle: 1}.is_immediate_number() &&
        Value {handle: 5}.is_immediate_number())
    }

    #[test]
    fn from_u64() {
        let x: u64 = 17;
        let y: Value = x.into();
        let z: u64 = y.into();
        assert_eq!(x, z)
    }
}
