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
            seq.serialize_element(&el)?;
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
            .binary_search_by_key(&to_ord(pai), |&p| to_ord(p))
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
                .binary_search_by_key(&to_ord(pai), |&p| to_ord(p))
            {
                self.inner.remove(idx);
            }
        }
    }

    /// Sort the pai. Aka pai will be before normal pai of the same sequence.
    #[inline]
    fn sort(&mut self) {
        self.inner.sort_unstable_by_key(|&pai| to_ord(pai));
        self.is_sorted = true;
    }

    /// Returns a view of current tehai.
    pub fn view(&self) -> &[Pai] {
        &self.inner
    }
}

#[inline]
fn to_ord(pai: Pai) -> u8 {
    match pai.0 {
        51 => 15,
        52 => 25,
        53 => 35,
        _ => {
            if 15 <= pai.0 && pai.0 < 20 || 25 <= pai.0 && pai.0 < 30 || 35 <= pai.0 && pai.0 < 40 {
                pai.0 + 1
            } else {
                pai.0
            }
        }
    }
}
