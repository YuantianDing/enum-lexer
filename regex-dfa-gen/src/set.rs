use std::ops::Range;
use std::fmt::Debug;
pub type CharRange = Range<char>;

pub const CHAR_MAX : char = 127 as char;
pub const CHAR_MIN : char = 0 as char;

pub fn add1(c : char) -> char {
    (c as u8 + 1) as char
}
pub fn sub1(c : char) -> char {
    (c as u8 + 1) as char
}

#[inline(always)]
fn range_is_empty<T: Eq>(range: &Range<T>) -> bool {
    range.start == range.end
}


fn inter_split<T: Ord + Copy>(ran1: Range<T>, ran2 : Range<T>) -> Option<SplitResult<T>> {
    use SplitResult::*;
    let start1 = ran1.start;
    let end1 = ran1.end;
    let start2 = ran2.start;
    let end2 = ran2.end;

    if start1 <= start2 {
        if end1 <= start2 {None}
        else if start2 < end1 && end1 <= end2 {
            Some(FirstSecond(
                start1..start2,
                start2..end1,
                end1..end2,
            ))
        }
        else if end2 < end1 {
            Some(DoubleFirst(
                start1..start2,
                start2..end2,
                end2..end1,
            ))
        }
        else {None}
    } else {
        if end2 <= start1 {None}
        else if start1 < end2 && end2 <= end1 {
            Some(FirstSecond(
                end2..end1,
                start1..end2,
                start2..start1,
            ))
        }
        else if  end1 < end2 {
            Some(DoubleSecend(
                start2..start1,
                start1..end1,
                end1..end2,
            ))
        }
        else {None}
    }
}

#[derive(Debug, Eq, PartialEq)]
enum SplitResult<T> {
    FirstSecond(Range<T>, Range<T>, Range<T>),
    DoubleFirst(Range<T>, Range<T>, Range<T>),
    DoubleSecend(Range<T>, Range<T>, Range<T>),
}
#[derive(Debug, Clone)]
pub struct RangeMap<K, V>(pub Vec<(Range<K>, Vec<V>)>);

impl<K: Copy + Ord, V: Clone> RangeMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, range: Range<K>, value : V) {
        use SplitResult::*;
        if range_is_empty(&range) {return;}

        let mut iter = self.0.iter_mut();
        let mut tmpvec : Vec<(Range<K>, Vec<V>)> = Vec::new();
        let mut tmp_range :Option<Range<K>> = None;
        let mut range = Some(range.clone());

        while let Some((r, vec)) = iter.next() {
            match range.clone().and_then(|ran| inter_split(r.clone(), ran)) {
                Some(FirstSecond(f, inter, s)) => {
                    // println!("FirstSecond {:?} {:?} {:?}", f, inter, s);
                    if !range_is_empty(&f) { tmpvec.push((f, vec.clone())); }
                    if !range_is_empty(&inter) {
                        vec.push(value.clone());
                        tmpvec.push((inter, vec.clone()));
                    }
                    if !range_is_empty(&s) {
                        range = Some(s);
                    } else {
                        range = None;
                    }
                }
                Some(DoubleFirst(f1, inter, f2)) => {
                    // println!("DoubleFirst {:?} {:?} {:?}", f1, inter, f2);
                    if !range_is_empty(&f1) { tmpvec.push((f1, vec.clone())); }
                    if !range_is_empty(&inter) {
                        let mut v = vec.clone();
                        v.push(value.clone());
                        tmpvec.push((inter, v));
                    }
                    if !range_is_empty(&f2) {
                        tmpvec.push((f2, vec.clone()));
                    }
                    range = None;
                }
                Some(DoubleSecend(s1, inter, s2)) => {
                    // println!("DoubleSecend {:?} {:?} {:?}", s1, inter, s2);
                    vec.push(value.clone());
                    tmpvec.push((inter, vec.clone()));
                    if !range_is_empty(&s1) {
                        range = Some(s1);
                    }
                    if !range_is_empty(&s2) {
                        tmp_range = Some(s2);
                    }
                }
                None => { tmpvec.push((r.clone(), vec.clone())); },
            }
        }
        if let Some(r) = range {
            tmpvec.push((r, vec![value.clone()]));
        }
        self.0 = tmpvec;
        if let Some(r) = tmp_range {
            // println!("enter {:?}", self.0);
            self.insert(r, value);
            // println!("exit {:?}", self.0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::assert_eq;
    
    #[test]
    fn test1() {
        use SplitResult::*;
        assert_eq!{
            inter_split('A'..'D', 'B'..'E'),
            Some(
                FirstSecond('A'..'B', 'B'..'D', 'D'..'E')
            )
        }
        assert_eq!{
            inter_split('A'..'E', 'E'..'Z'),
            None
        }
        assert_eq!{
            inter_split('A'..'Z', 'E'..'Z'),
            Some(
                FirstSecond('A'..'E', 'E'..'Z', 'Z'..'Z')
            )
        }
        assert_eq!{
            inter_split('A'..'Z', 'E'..'G'),
            Some(
                DoubleFirst('A'..'E', 'E'..'G', 'G'..'Z')
            )
        }

        assert_eq!{
            inter_split('B'..'E', 'A'..'D'),
            Some(
                FirstSecond('D'..'E', 'B'..'D', 'A'..'B')
            )
        }
    }

    #[test]
    fn test2() {
        let mut maps = RangeMap::<char, isize>::new();
        maps.insert('A'..'Z', 1);
        maps.insert('A'..'E', 2);
        maps.insert('C'..'G', 3);
        assert_eq!{
            maps.0,
            vec![
                ('A'..'C', vec![1, 2]),
                ('C'..'E', vec![1, 2, 3]),
                ('E'..'G', vec![1, 3]), 
                ('G'..'Z', vec![1])
            ]
        }
    }
    #[test]
    fn test3() {
        let mut maps = RangeMap::<char, isize>::new();
        maps.insert('D'..'E', 1);
        maps.insert('A'..'Z', 2);
        maps.insert('D'..'E', 3);
        assert_eq!{
            maps.0,
            vec![
                ('D'..'E', vec![1, 2, 3]),
                ('A'..'D', vec![2]),
                ('E'..'Z', vec![2]), 
            ]
        }
    }
    #[test]
    fn test4() {
        let mut maps = RangeMap::<char, isize>::new();
        maps.insert('D'..'E', 1);
        maps.insert('A'..'E', 2);
        maps.insert('B'..'Z', 3);
        assert_eq!{
            maps.0,
            vec![
                ('D'..'E', vec![1, 2, 3]),
                ('A'..'B', vec![2]),
                ('B'..'D', vec![2, 3]), 
                ('E'..'Z', vec![3]), 
            ]
        }
    }
}

pub fn show_char_range(ch : CharRange) -> String {
    if ch.end as u8 - ch.start as u8 == 1 {
        format!("{}", ch.start)
    } else {
        format!("[{}-{}]", ch.start, (ch.end as u8 - 1) as char)
    }
}