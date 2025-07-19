pub trait PartialTreeEq {
    type Other: PartialTreeEq;
    fn partial_eq(&self, other: &Self::Other) -> bool;
}



pub fn partial_eq<T: PartialTreeEq<Other = T>>(
    a: &T, b: &T
) -> bool {
    a.partial_eq(b)
}

pub fn partial_eq_all<T: PartialTreeEq<Other = T>>(
    a: &Vec<T>, b: &Vec<T>
) -> bool {
    if  a.len() != b.len() {
        return false;
    }
    
    let ab = a.iter()
        .zip(b.iter());

    for (a, b) in ab {
        if !a.partial_eq(b) {
            return false;
        }
    }

    true
}