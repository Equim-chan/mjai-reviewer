use convlog::Tile;

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Default)]
pub struct Tehai {
    inner: Vec<Tile>,
    is_sorted: bool,
}

impl From<Vec<Tile>> for Tehai {
    #[inline]
    fn from(tehai: Vec<Tile>) -> Self {
        let mut ret = Self {
            inner: tehai,
            is_sorted: false,
        };
        ret.sort();
        ret
    }
}

impl From<Tehai> for Vec<Tile> {
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
        self.inner.serialize(serializer)
    }
}

impl Tehai {
    /// Resets current tehai.
    #[inline]
    pub fn haipai(&mut self, tiles: &[Tile]) {
        self.inner = tiles.to_vec();
        self.sort();
    }

    /// Tsumo a tile.
    #[inline]
    pub fn tsumo(&mut self, tile: Tile) {
        self.inner.push(tile);
        self.is_sorted = false;
    }

    /// Tsumogiri a tile.
    #[inline]
    pub fn tsumogiri(&mut self) {
        self.inner.pop();
        self.is_sorted = true;
    }

    /// Tedashi a tile.
    pub fn tedashi(&mut self, tile: Tile) {
        if !self.is_sorted {
            self.sort();
        }

        if let Ok(idx) = self.inner.binary_search(&tile) {
            self.inner.remove(idx);
        }
    }

    /// Remove several tiles for fuuro.
    pub fn remove_multiple(&mut self, tiles: &[Tile]) {
        // usually, it is already sorted, except for kakan and ankan.
        if !self.is_sorted {
            self.sort();
        }

        for &tile in tiles {
            if let Ok(idx) = self.inner.binary_search(&tile) {
                self.inner.remove(idx);
            }
        }
    }

    /// Sort the tiles. Aka tile will be before normal tile of the same
    /// sequence.
    #[inline]
    fn sort(&mut self) {
        self.inner.sort_unstable();
        self.is_sorted = true;
    }

    /// Returns a view of current tehai.
    pub fn view(&self) -> &[Tile] {
        &self.inner
    }
}
