use bitvec::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitDomain {
    bits: BitVec<u32, Lsb0>,
}

impl BitDomain {
    pub fn empty(n: u8) -> Self {
        Self {
            bits: bitvec![u32, Lsb0; 0; n as usize + 1],
        }
    }

    pub fn full(n: u8) -> Self {
        let mut d = Self::empty(n);
        for v in 1..=n {
            d.insert(v);
        }
        d
    }

    pub fn insert(&mut self, v: u8) {
        if let Some(mut bit) = self.bits.get_mut(v as usize) {
            *bit = true;
        }
    }

    pub fn remove(&mut self, v: u8) {
        if let Some(mut bit) = self.bits.get_mut(v as usize) {
            *bit = false;
        }
    }

    pub fn contains(&self, v: u8) -> bool {
        self.bits.get(v as usize).as_deref() == Some(&true)
    }

    pub fn count(&self) -> u32 {
        self.bits.count_ones() as u32
    }

    pub fn iter_values(&self) -> impl Iterator<Item = u8> + '_ {
        self.bits.iter_ones().skip(1).map(|idx| idx as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_insert_remove() {
        let mut d = BitDomain::full(6);
        assert!(d.contains(6));
        d.remove(6);
        assert!(!d.contains(6));
        d.insert(6);
        assert!(d.contains(6));
        assert_eq!(d.count(), 6);
    }
}
