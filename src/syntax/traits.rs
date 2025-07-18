pub trait PartialTreeEq {
    type Other: PartialTreeEq;
    fn partial_eq(&self, other: &Self::Other) -> bool;
}



pub fn partial_eq<T: PartialTreeEq<Other = T>>(
    a: &T, b: &T
) -> bool {
    a.partial_eq(b)
}