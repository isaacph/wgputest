
pub trait PartialOrdMinMax<P: PartialOrd> {
    fn partial_max(self) -> Option<P>;
    fn partial_min(self) -> Option<P>;
}

impl<T, P> PartialOrdMinMax<P> for T
            where P: PartialOrd,
                  T: Iterator<Item = P> {
    fn partial_max(self) -> Option<P> {
        use std::cmp::Ordering::*;
        self.fold(None, |cur, next| {
            match cur {
                None => Some(next),
                Some(cur) => match cur.partial_cmp(&next) {
                    None => None,
                    Some(Less) => Some(next),
                    Some(Equal) | Some(Greater) => Some(cur),
                },
            }
        })
    }

    fn partial_min(self) -> Option<P> {
        use std::cmp::Ordering::*;
        self.fold(None, |cur, next| {
            match cur {
                None => Some(next),
                Some(cur) => match cur.partial_cmp(&next) {
                    None => None,
                    Some(Greater) => Some(next),
                    Some(Equal) | Some(Less) => Some(cur),
                },
            }
        })
    }
}
