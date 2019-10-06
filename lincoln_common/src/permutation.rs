use core::fmt::{Display, Error, Formatter};
use core::str::FromStr;
use smallvec::SmallVec;

/// A trait to treat some suitable type to a permutation, without consuming
pub trait AsPermutation {
    //Return a permutation from a shared reference
    fn as_permutation(&self) -> Result<Permutation, failure::Error>;
}

pub const FACTS: [u64; 20] = [
    1,
    2,
    6,
    24,
    120,
    720,
    5_040,
    40_320,
    362_880,
    3_628_800,
    79_833_600,
    958_003_200,
    6_227_020_800,
    87_178_291_200,
    174_356_582_400,
    2_615_348_736_000,
    41_845_579_776_000,
    711_374_856_192_000,
    12_804_747_411_456_000,
    243_290_200_817_664_000,
];

/// Represents a permutation of up to 20 positions.
/// The representation is the following:
/// 
/// 1. Decide whether the value at position 1 should be in position 0.
/// 2. If yes, a swap (0<->1) is needed.
/// 3. Decide whether the value at position 2 should be in position 0 or 1.
/// 4. If yes, perform (0<->2) or (1<->2) accordingly
/// 5. Repeat until we reach the end of the permutation.
/// 6. We now have a list of permutation to perform. 
/// 7. We then encode (m<->n) as (m+1)*n!, and add them together to get the result.
/// 
/// It is provable that the number we get for the above are unique for permutations.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Permutation(pub u64);
impl Display for Permutation {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let mut v = *b"abcdefghijklmnopqrst";
        let r = &mut v[0..self.min_len() as usize];
        self.permutate(r);
        write!(
            fmt,
            "{}",
            std::str::from_utf8(r).map_err(|_| Error::default())?
        )
    }
}
impl From<&Permutation> for String {
    fn from(s: &Permutation) -> String {
        format!("{}", s)
    }
}
impl<T> AsPermutation for T
where
    T: AsRef<str>,
{
    fn as_permutation(&self) -> Result<Permutation, failure::Error> {
        Permutation::from_str(self.as_ref())
    }
}
impl AsPermutation for Permutation {
    fn as_permutation(&self) -> Result<Permutation, failure::Error> {
        Ok(*self)
    }
}
impl AsPermutation for &Permutation {
    fn as_permutation(&self) -> Result<Permutation, failure::Error> {
        Ok(**self)
    }
}
impl Permutation {
    /// Perform the permutation on a given set of values.
    /// 
    /// values: the values to permutate
    /// 
    pub fn permutate<T>(self, values: &mut [T]) {
        let mut v = self.0;
        for i in 1..values.len() {
            let r = v % (i + 1) as u64;
            if r > 0 {
                values.swap((r - 1) as usize, i)
            };
            v /= (i + 1) as u64;
        }
    }
    fn identical() -> Permutation {
        Permutation(0)
    }
    /// Returns a permucation that swap position i and j.
    /// 
    /// We limit the maximum number positions to 255,
    /// because our underline `u64` type only supports 
    /// 20 positions and anything greater than that does not
    /// make sende.
    pub fn swap(i: u8, j: u8) -> Permutation {
        if j == 0 {
            return Self::identical();
        }
        let (i, j) = (u64::from(i), u64::from(j));
        Permutation(FACTS[(i + j - 1) as usize] * (i + 1))
    }
    /// Returns the minimal number of positions to perform the permutation
    /// 
    /// returns: the required number of positions
    pub fn min_len(self) -> u8 {
        match FACTS.binary_search_by(|v| v.cmp(&self.0)) {
            Ok(n) => (n + 2) as u8,
            Err(0) => 0,
            Err(n) => (n + 1) as u8,
        }
    }
}
impl From<u64> for Permutation {
    fn from(v: u64) -> Self {
        Self(v)
    }
}
impl FromStr for Permutation {
    type Err = failure::Error;
    fn from_str(s: &str) -> Result<Permutation, Self::Err> {
        let bs = s.as_bytes();
        if bs.len() > 20 {
            bail!("permutation string too long");
        }
        let mut v: SmallVec<[u8; 20]> = smallvec![];
        v.extend_from_slice(bs);
        let mut p = 0;
        while !v.is_empty() {
            let c = (b'a' as usize + v.len() - 1) as u8;
            let idx = v.iter().position(|v| *v == c);
            let idx = match idx {
                Some(idx) => idx,
                None => bail!("character {} not found", c - b'a'),
            };
            if idx < v.len() - 1 {
                let p1 = Permutation::swap(idx as u8, (v.len() - idx - 1) as u8);
                p += p1.0;
                p1.permutate(&mut v);
            }
            let _ = v.pop();
        }
        Ok(Permutation(p))
    }
}

#[cfg(test)]
mod test {
    use crate::permutation::Permutation;
    use std::str::FromStr;
    #[test]
    fn test_parse() {
        assert_eq!("ba".parse::<Permutation>().unwrap(), Permutation(1));
    }

    #[test]
    fn test_swap() {
        assert_eq!("ba", format!("{}", Permutation(1)));

        assert_eq!(Permutation::swap(0, 1), Permutation(1));
        assert_eq!(Permutation::swap(0, 2), Permutation(2));
        assert_eq!(Permutation::swap(1, 1), Permutation(4));
        assert_eq!(Permutation::swap(0, 3), Permutation(6));
        assert_eq!(Permutation::swap(1, 2), Permutation(12));
        assert_eq!(Permutation::swap(2, 1), Permutation(18));

        assert_eq!(Permutation::swap(0, 4), Permutation(24));
        assert_eq!(Permutation::swap(0, 3), Permutation(6));
        assert_eq!(Permutation::swap(1, 1), Permutation(4));
        assert_eq!(Permutation::swap(0, 1), Permutation(1));

        assert_eq!(format!("{}", Permutation(35)), "ecabd");
        assert_eq!(format!("{}", Permutation(82)), "dceab");
        assert_eq!(format!("{}", Permutation(17)), "bdac");
        assert_eq!(format!("{}", Permutation(2)), "cba");
        assert_eq!(format!("{}", Permutation(4)), "acb");

        assert_eq!(Permutation::from_str("ecabd").unwrap(), Permutation(35));
        assert_eq!(Permutation::from_str("dceab").unwrap(), Permutation(82));
        assert_eq!(Permutation::from_str("bdac").unwrap(), Permutation(17));
    }
    #[test]
    fn test_permutation() {
        let mut v = [1, 2, 3, 4];
        Permutation(1).permutate(&mut v); //swap(0,1)
        assert_eq!(v, [2, 1, 3, 4]);
        Permutation(2).permutate(&mut v); //swap(0,2)
        assert_eq!(v, [3, 1, 2, 4]);
        Permutation(4).permutate(&mut v); //swap(0,3)
        assert_eq!(v, [3, 2, 1, 4]);
    }
}
