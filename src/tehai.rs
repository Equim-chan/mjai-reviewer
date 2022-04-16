use convlog::Pai;

use serde::ser::{Serialize, SerializeSeq, Serializer};

#[derive(Debug, Clone, Default)]
pub struct Tehai {
    inner: Vec<Pai>,
    is_sorted: bool,
}

impl From<Vec<Pai>> for Tehai {
    #[inline]
    fn from(tehai: Vec<Pai>) -> Self {
        let mut ret = Self {
            inner: tehai,
            is_sorted: false,
        };
        ret.sort();
        ret
    }
}

impl From<Tehai> for Vec<Pai> {
    #[inline]
    fn from(tehai: Tehai) -> Self {
        tehai.inner
    }
}

impl Serialize for Tehai {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.inner.len()))?;
        for el in &self.inner {
            seq.serialize_element(&el.to_string())?;
        }
        seq.end()
    }
}

impl Tehai {
    /// Resets current tehai.
    #[inline]
    pub fn haipai(&mut self, pais: &[Pai]) {
        self.inner = pais.to_vec();
        self.sort();
    }

    /// Tsumo a pai.
    #[inline]
    pub fn tsumo(&mut self, pai: Pai) {
        self.inner.push(pai);
        self.is_sorted = false;
    }

    /// Tsumogiri a pai.
    #[inline]
    pub fn tsumogiri(&mut self) {
        self.inner.pop();
        self.is_sorted = true;
    }

    /// Tedashi a pai.
    pub fn tedashi(&mut self, pai: Pai) {
        if !self.is_sorted {
            self.sort();
        }

        if let Ok(idx) = self
            .inner
            .binary_search_by_key(&pai.as_ord(), |pai| pai.as_ord())
        {
            self.inner.remove(idx);
        }
    }

    /// Remove several pais for fuuro.
    pub fn remove_multiple(&mut self, pais: &[Pai]) {
        // usually, it is already sorted, except for kakan and ankan.
        if !self.is_sorted {
            self.sort();
        }

        for &pai in pais {
            if let Ok(idx) = self
                .inner
                .binary_search_by_key(&pai.as_ord(), |pai| pai.as_ord())
            {
                self.inner.remove(idx);
            }
        }
    }

    /// Sort the pai. Aka pai will be before normal pai of the same sequence.
    #[inline]
    fn sort(&mut self) {
        self.inner.sort_unstable_by_key(|pai| pai.as_ord());
        self.is_sorted = true;
    }

    /// Returns a view of current tehai.
    pub fn view(&self) -> &[Pai] {
        &self.inner
    }

    // Format into tenhou format
    // e.g. "123789m789p2355s1s"
    pub fn as_tenhou_str(&self) -> String {
        let pais = if self.is_sorted {
            self.inner.clone()
        } else {
            let mut pais = self.inner.clone();
            pais.sort_unstable_by_key(|pai| pai.as_ord());
            pais
        };
        let get_suffix = |p: &Pai| match p.as_u8() {
            11..=19 | 51 => 'm',
            21..=29 | 52 => 'p',
            31..=39 | 53 => 's',
            41..=47 => 'z',
            _ => unreachable!(),
        };
        let zero_ord = 48u8;
        let mut res = String::new();
        let mut cur_suffix = get_suffix(&pais[0]);
        for p in pais {
            let p_as_u8 = p.as_u8();
            let match_suffix = match p_as_u8 {
                11..=19 | 51 => cur_suffix == 'm',
                21..=29 | 52 => cur_suffix == 'p',
                31..=39 | 53 => cur_suffix == 's',
                41..=47 => cur_suffix == 'z',
                _ => unreachable!(),
            };
            if !match_suffix {
                res.push(cur_suffix);
            }
            cur_suffix = get_suffix(&p);
            match p_as_u8 {
                51 | 52 | 53 => res.push('0'),
                11..=19 => res.push((zero_ord + p_as_u8 - 10) as char),
                21..=29 => res.push((zero_ord + p_as_u8 - 20) as char),
                31..=39 => res.push((zero_ord + p_as_u8 - 30) as char),
                41..=47 => res.push((zero_ord + p_as_u8 - 40) as char),
                _ => unreachable!(),
            }
        }
        res.push(cur_suffix);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use convlog::pai::get_pais_from_str;

    #[test]
    fn test_tehai() {
        let tehai = Tehai::from(get_pais_from_str("123456789p123s55m").unwrap());
        assert_eq!("55m123456789p123s", tehai.as_tenhou_str());
        let tehai = Tehai::from(get_pais_from_str("123406789p406s05m").unwrap());
        assert_eq!("50m123406789p406s", tehai.as_tenhou_str());
    }
}
