use crate::tehai::Tehai;
use crate::{log, log_if};

use anyhow::anyhow;
use anyhow::{Context, Result};
use convlog::mjai::{Consumed2, Consumed3, Consumed4, Event};
use convlog::Pai;
use itertools::{EitherOrBoth::*, Itertools};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::iter::FromIterator;

#[derive(Debug, Clone, Default, Serialize)]
pub struct State {
    #[serde(skip)]
    actor: u8,

    pub tehai: Tehai,
    pub fuuros: Vec<Fuuro>,
}

struct PaiIterator<'a> {
    tehai: std::slice::Iter<'a, convlog::Pai>,
    fuuros: std::slice::Iter<'a, Fuuro>,
    cur_fuuro: Option<Vec<Pai>>,
    cur_fuuro_index: usize,
}

impl<'a> Iterator for PaiIterator<'a> {
    type Item = Pai;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pai) = self.tehai.next() {
            return Some(*pai);
        }

        // get pai from current fuuro
        if let Some(curr) = &self.cur_fuuro {
            if self.cur_fuuro_index < curr.len() {
                let pai = curr[self.cur_fuuro_index];
                self.cur_fuuro_index += 1;
                return Some(pai);
            }
        }
        // get pai from next fuuro
        if let Some(next_fuuro) = self.fuuros.next() {
            let pais = next_fuuro.into_pais();
            let first_pai = pais[0];
            self.cur_fuuro = Some(pais);
            self.cur_fuuro_index = 1;
            return Some(first_pai);
        }
        // nothing left
        None
    }
}

impl State {
    #[inline]
    pub fn new(actor: u8) -> Self {
        State {
            actor,
            ..Self::default()
        }
    }

    /// Argument `event` must be one of
    ///
    /// * StartKyoku
    /// * Tsumo
    /// * Dahai
    /// * Chi
    /// * Pon
    /// * Kakan
    /// * Daiminkan
    /// * Ankan
    ///
    /// and the `actor` must be the target actor.
    ///
    /// Otherwise this is a no-op.
    pub fn update(&mut self, event: &Event) -> Result<()> {
        match *event {
            Event::StartKyoku { tehais, .. } => {
                self.tehai.haipai(&tehais[self.actor as usize]);
                self.fuuros.clear();
            }

            Event::Tsumo { actor, pai } if actor == self.actor => self.tehai.tsumo(pai),

            Event::Dahai {
                actor,
                pai,
                tsumogiri,
            } if actor == self.actor => {
                if tsumogiri {
                    self.tehai.tsumogiri();
                } else {
                    self.tehai.tedashi(pai);
                }
            }

            Event::Chi {
                actor,
                target,
                pai,
                consumed,
            } if actor == self.actor => {
                self.tehai.remove_multiple(&consumed.as_array());

                let fuuro = Fuuro::Chi {
                    target,
                    pai,
                    consumed,
                };
                self.fuuros.push(fuuro);
            }

            Event::Pon {
                actor,
                target,
                pai,
                consumed,
            } if actor == self.actor => {
                self.tehai.remove_multiple(&consumed.as_array());

                let fuuro = Fuuro::Pon {
                    target,
                    pai,
                    consumed,
                };
                self.fuuros.push(fuuro);
            }

            Event::Daiminkan {
                actor,
                target,
                pai,
                consumed,
            } if actor == self.actor => {
                self.tehai.remove_multiple(&consumed.as_array());

                let fuuro = Fuuro::Daiminkan {
                    target,
                    pai,
                    consumed,
                };
                self.fuuros.push(fuuro);
            }

            Event::Kakan {
                actor,
                pai,
                consumed,
            } if actor == self.actor => {
                self.tehai.tedashi(pai);

                let (
                    previous_pon_idx,
                    previous_pon_target,
                    previous_pon_pai,
                    previous_pon_consumed,
                ) = self
                    .fuuros
                    .iter()
                    .enumerate()
                    .find_map(|(idx, f)| match *f {
                        Fuuro::Pon {
                            target: pon_target,
                            pai: pon_pai,
                            consumed: pon_consumed,
                        } if Consumed3::from([
                            pon_pai,
                            pon_consumed.as_array()[0],
                            pon_consumed.as_array()[1],
                        ]) == consumed =>
                        {
                            Some((idx, pon_target, pon_pai, pon_consumed))
                        }

                        _ => None,
                    })
                    .context(anyhow!("invalid state: previous Pon not found for Kakan"))?;

                let fuuro = Fuuro::Kakan {
                    pai,
                    previous_pon_target,
                    previous_pon_pai,
                    consumed: previous_pon_consumed,
                };
                self.fuuros[previous_pon_idx] = fuuro;
            }

            Event::Ankan { actor, consumed } if actor == self.actor => {
                self.tehai.remove_multiple(&consumed.as_array());

                let fuuro = Fuuro::Ankan { consumed };
                self.fuuros.push(fuuro);
            }

            _ => (),
        };

        Ok(())
    }

    fn iter(&self) -> PaiIterator {
        PaiIterator {
            tehai: self.tehai.view().iter(),
            fuuros: self.fuuros.iter(),
            cur_fuuro: None,
            cur_fuuro_index: 0,
        }
    }

    // calculate the shanten
    pub fn calc_shanten(&self) -> i32 {
        let mut s = ShantenHelper::new(&self.tehai);
        s.get_shanten()
    }
}

#[serde_as]
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Fuuro {
    Chi {
        target: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        consumed: Consumed2,
    },
    Pon {
        target: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        consumed: Consumed2,
    },
    Daiminkan {
        target: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        consumed: Consumed3,
    },
    Kakan {
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        previous_pon_target: u8,
        #[serde_as(as = "DisplayFromStr")]
        previous_pon_pai: Pai,
        consumed: Consumed2,
    },
    Ankan {
        consumed: Consumed4,
    },
}

impl Fuuro {
    fn into_pais(&self) -> Vec<Pai> {
        let mut return_pais: Vec<Pai> = Vec::new();
        match self {
            Self::Chi { pai, consumed, .. } => {
                return_pais.push(*pai);
                return_pais.extend_from_slice(&consumed.as_array());
            }
            Self::Pon { pai, consumed, .. } => {
                return_pais.push(*pai);
                return_pais.extend_from_slice(&consumed.as_array());
            }
            Self::Daiminkan { pai, consumed, .. } => {
                return_pais.push(*pai);
                return_pais.extend_from_slice(&consumed.as_array());
            }
            Self::Kakan {
                pai,
                previous_pon_pai,
                consumed,
                ..
            } => {
                return_pais.push(*pai);
                return_pais.push(*previous_pon_pai);
                return_pais.extend_from_slice(&consumed.as_array());
            }
            Self::Ankan { consumed } => {
                return_pais.extend_from_slice(&consumed.as_array());
            }
        }
        return_pais
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Distance {
    One,
    Two,
    Inf,
}
impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Distance::Inf => '∞',
                &Distance::One => '1',
                &Distance::Two => '2',
            }
        )
    }
}

#[derive(Debug)]
struct BlockElem {
    pai: Pai,
    num: u32,
    // The distance between current Pai and next Pai
    distance: Distance,
}
type Block = Vec<BlockElem>;

struct ShantenHelper {
    blocks: Vec<Block>,
    num_pais_rem: i32,
    num_tehai: i32,

    verbose: bool,
    state: Vec<Vec<Pai>>,
}

impl ShantenHelper {
    fn new(tehai: &Tehai) -> Self {
        // collect all pais in tehai and fuuros
        let (pai_counter, num_pais) = {
            let mut pai_counter: BTreeMap<u8, u32> = BTreeMap::new();
            let mut num_pais = 0;
            for pai in tehai.view().iter() {
                // Pai.as_unify_u8 has handled 0m 0p 0s => 5m 5p 5s
                num_pais += 1;
                *pai_counter.entry(pai.as_unify_u8()).or_insert(0) += 1;
            }
            (pai_counter, num_pais)
        };
        // assert!(num_pais == 14 || num_pais == 13);
        let pai_counter_vec = Vec::from_iter(pai_counter.iter());

        let get_pai_type = |pai: u8| {
            match pai {
                11..=19 => 1, // m
                21..=29 => 2, // p
                31..=39 => 3, // s
                41..=47 => 4, // z
                x => unreachable!(format!("with pai: {}", x)),
            }
        };
        let distance: Vec<Distance> = pai_counter_vec
            .windows(2)
            .map(|pais| {
                let (pai, next_pai) = (pais[0].0, pais[1].0);
                let (pai_type, next_pai_type) = (get_pai_type(*pai), get_pai_type(*next_pai));
                if pai_type != next_pai_type || pai_type == 4 {
                    // if the adj pais are from different type, or both of them are "z",
                    // then the distance between them are ∞
                    Distance::Inf
                } else if next_pai - pai > 2 {
                    Distance::Inf // ∞
                } else if next_pai - pai == 1 {
                    Distance::One
                } else {
                    Distance::Two
                }
            })
            .collect();

        let mut blocks = Vec::new();
        let mut cur_block: Block = Vec::new();
        for pair in pai_counter_vec.iter().zip_longest(distance.iter()) {
            match pair {
                Both(p, d) => {
                    let elem = BlockElem {
                        pai: Pai::try_from(*p.0).unwrap(),
                        num: *p.1,
                        distance: *d,
                    };
                    cur_block.push(elem);
                    match d {
                        &Distance::Inf => {
                            blocks.push(cur_block);
                            cur_block = Vec::new()
                        }
                        _ => {}
                    };
                }
                Left((p, num)) => {
                    // to the end
                    let elem = BlockElem {
                        pai: Pai::try_from(**p).unwrap(),
                        num: **num,
                        distance: Distance::Inf,
                    };
                    cur_block.push(elem);
                    blocks.push(cur_block);
                    cur_block = Vec::new()
                }
                Right(_) => unreachable!("the size of distance must less than pai_counter"),
            }
        }
        Self {
            blocks,
            num_pais_rem: num_pais,
            num_tehai: num_pais,
            verbose: false,
            state: Vec::new(),
        }
    }

    fn get_shanten(&mut self) -> i32 {
        let kokushi = self.get_kokushi_shanten();
        let chiitoi = self.get_chiitoi_shanten();
        let normal = self.get_normal_shanten();
        std::cmp::min(std::cmp::min(kokushi, chiitoi), normal)
    }

    // Get kokushi
    fn get_kokushi_shanten(&self) -> i32 {
        if self.num_tehai < 13 {
            return 8;
        }
        let mut num_kind = 0;
        let mut exist_pair = false;
        self.blocks.iter().for_each(|block| {
            for e in block.iter() {
                num_kind += match e.pai {
                    Pai::East
                    | Pai::South
                    | Pai::West
                    | Pai::North
                    | Pai::Chun
                    | Pai::Haku
                    | Pai::Hatsu
                    | Pai::Man1
                    | Pai::Man9
                    | Pai::Pin1
                    | Pai::Pin9
                    | Pai::Sou1
                    | Pai::Sou9 => {
                        if e.num > 1 {
                            exist_pair = true;
                        }
                        1
                    }
                    _ => 0,
                };
            }
        });
        13 - num_kind - if exist_pair { 1 } else { 0 }
    }

    // Get shanten for (7 * pair)
    fn get_chiitoi_shanten(&self) -> i32 {
        let mut shanten = 6i32; // 6 at max for chiitoi
                                // there is any fuuro, then we can not get chiitoi
        if self.num_tehai < 13 {
            return shanten;
        }
        let mut num_kind = 0;
        self.blocks.iter().for_each(|block| {
            for e in block.iter() {
                if e.num == 0 {
                    continue;
                }
                if e.num >= 2 {
                    shanten -= 1;
                }
                num_kind += 1;
            }
        });
        shanten += std::cmp::max(0, 7 - num_kind);
        shanten
    }

    // Get shanten for (4 * triple + 1 * eye)
    // Return shanten: i32. Range from [-1, 8].
    //    0 for tenpai
    //   -1 for ron
    fn get_normal_shanten(&mut self) -> i32 {
        log_if!(self.verbose, "num of blocks: {}", self.blocks.len());
        let mut shanten = 8i32;
        let mut c_max = 0i32;
        let k = (self.num_pais_rem - 2) / 3;
        let eye_candidates = ShantenHelper::eyes(&self.blocks);
        for eye in eye_candidates {
            // try to get the shanten with this eye
            log_if!(self.verbose, "take {} as eye begin", eye);
            self.take_eye(eye);
            self.search_by_take_3(0, &mut shanten, &mut c_max, k, 1, 0);
            self.rollback_pais(&[eye, eye]);
            log_if!(self.verbose, "take {} as eye done, s: {}", eye, shanten);
        }
        // try to get the shanten without eye
        log_if!(self.verbose, "take nothing as eye begin");
        self.search_by_take_3(0, &mut shanten, &mut c_max, k, 0, 0);
        log_if!(self.verbose, "take nothing as eye done, s: {}", shanten);
        shanten
    }

    fn eyes(blocks: &Vec<Block>) -> Vec<Pai> {
        blocks
            .iter()
            .flat_map(|block| block.iter())
            .filter(|elem| elem.num >= 2)
            .map(|elem| elem.pai)
            .collect()
    }

    fn take<const LEN: usize>(&mut self, pais: &[Pai; LEN]) {
        self.num_pais_rem -= pais.len() as i32;
        self.state.push(pais.to_vec());
    }

    fn rollback_pais<const LEN: usize>(&mut self, pais: &[Pai; LEN]) {
        for block in self.blocks.iter_mut() {
            for elem in block.iter_mut() {
                for p in pais {
                    if *p == elem.pai {
                        elem.num += 1;
                    }
                }
            }
        }
        self.num_pais_rem += pais.len() as i32;
        self.state.pop();
    }

    fn take_eye(&mut self, pai: Pai) {
        for block in self.blocks.iter_mut() {
            for elem in block.iter_mut() {
                if elem.pai == pai {
                    elem.num -= 2;
                    let eye = [elem.pai, elem.pai];
                    self.take(&eye);
                    return;
                }
            }
        }
    }

    fn try_take_3(&mut self, mut take_idx: i32) -> Option<[Pai; 3]> {
        // try to get triplet (AAA)
        for block in self.blocks.iter_mut() {
            for elem in block {
                if elem.num >= 3 {
                    if take_idx > 0 {
                        take_idx -= 1;
                    } else {
                        elem.num -= 3;
                        let meld = [elem.pai, elem.pai, elem.pai];
                        self.take(&meld);
                        return Some(meld);
                    }
                }
            }
        }
        // try to get sequence (ABC)
        for block in self.blocks.iter_mut() {
            if block.len() < 3 {
                continue;
            }
            for idx in (0..block.len()).step_by(3) {
                let p1 = &block[idx];
                if p1.num == 0 || p1.distance != Distance::One {
                    continue;
                }
                let p2 = &block[idx + 1];
                if p2.num == 0 || p2.distance != Distance::One {
                    continue;
                }
                let p3 = &block[idx + 2];
                if p3.num == 0 {
                    continue;
                } else {
                    if take_idx > 0 {
                        take_idx -= 1;
                    } else {
                        let meld = [p1.pai, p2.pai, p3.pai];
                        block[idx].num -= 1;
                        block[idx + 1].num -= 1;
                        block[idx + 2].num -= 1;
                        self.take(&meld);
                        return Some(meld);
                    }
                }
            }
        }
        None
    }

    fn try_take_2(&mut self, mut take_idx: i32) -> Option<[Pai; 2]> {
        // try get pair
        for block in self.blocks.iter_mut() {
            for elem in block.iter_mut() {
                if elem.num >= 2 {
                    if take_idx > 0 {
                        take_idx -= 1;
                    } else {
                        elem.num -= 2;
                        let res = [elem.pai, elem.pai];
                        self.take(&res);
                        return Some(res);
                    }
                }
            }
        }
        // try get RYANMEN/PENCHAN/KANCHAN
        for block in self.blocks.iter_mut() {
            if block.len() < 2 {
                continue;
            }
            for idx in 0..block.len() {
                let p1 = &block[idx];
                if p1.num == 0 || idx == block.len() - 1 {
                    continue;
                }
                let p2 = &block[idx + 1];
                if p2.num > 0 {
                    // RYANMEN/PENCHAN
                    if take_idx > 0 {
                        take_idx -= 1;
                    } else {
                        let res = [p1.pai, p2.pai];
                        block[idx].num -= 1;
                        block[idx + 1].num -= 1;
                        self.take(&res);
                        return Some(res);
                    }
                }
                if p1.distance == Distance::Two
                    || p2.distance == Distance::Two
                    || idx == block.len() - 2
                {
                    continue;
                }
                // KANCHAN
                let p3 = &block[idx + 2];
                if p3.num > 0 {
                    if take_idx > 0 {
                        take_idx -= 1;
                    } else {
                        let res = [p1.pai, p3.pai];
                        block[idx].num -= 1;
                        block[idx + 2].num -= 1;
                        self.take(&res);
                        return Some(res);
                    }
                }
            }
        }
        None
    }

    fn try_take_1(&mut self, mut take_idx: i32) -> Option<Pai> {
        for block in self.blocks.iter_mut() {
            for elem in block.iter_mut() {
                if elem.num > 0 {
                    if take_idx > 0 {
                        take_idx -= 1;
                    } else {
                        elem.num -= 1;
                        let pai = elem.pai;
                        self.take(&[pai]);
                        return Some(pai);
                    }
                }
            }
        }
        None
    }

    fn search_by_take_3(
        &mut self,
        i: i32,
        shanten: &mut i32,
        c_max: &mut i32,
        k: i32,
        exist_eye: i32,
        num_meld: i32,
    ) {
        log_if!(
            self.verbose,
            "entry search_by meld with i: {}, c_rem: {}, s: {}, c_max: {}",
            i,
            self.num_pais_rem,
            shanten,
            c_max
        );
        if i == self.num_pais_rem {
            self.search_by_take_2(0, shanten, c_max, k, exist_eye, num_meld, 0);
            return;
        }

        // take a meld
        if let Some(meld) = self.try_take_3(i) {
            log_if!(self.verbose, "take {:?} as meld begin", meld);
            self.search_by_take_3(i, shanten, c_max, k, exist_eye, num_meld + 1);
            self.rollback_pais(&meld);
            log_if!(
                self.verbose,
                "take {:?} as meld done, s: {}",
                meld,
                *shanten
            );
        }
        log_if!(self.verbose, "take nothing as meld begin");
        self.search_by_take_3(i + 1, shanten, c_max, k, exist_eye, num_meld);
        log_if!(self.verbose, "take nothing as meld done, s: {}", *shanten);
    }

    fn search_by_take_2(
        &mut self,
        i: i32,
        shanten: &mut i32,
        c_max: &mut i32,
        k: i32,
        exist_eye: i32,
        num_meld: i32,
        num_dazi: i32,
    ) {
        log_if!(
            self.verbose,
            "entry search_by 2 with i: {}, c_rem: {}, s: {}, c_max: {}, g: {}, gp: {}",
            i,
            self.num_pais_rem,
            shanten,
            c_max,
            num_meld,
            num_dazi
        );
        if *shanten == -1 || num_meld + num_dazi > self.num_tehai {
            log_if!(
                self.verbose,
                "search end. cur state: {:?}. cause: s: {}, {} + {} > {}",
                self.state,
                shanten,
                num_meld,
                num_dazi,
                self.num_tehai
            );
            return;
        }
        let c = 3 * num_meld + 2 * num_dazi + 2 * exist_eye;
        if self.num_pais_rem < *c_max - c {
            log_if!(
                self.verbose,
                "search end. cur state: {:?}. cause: {} < {} - {}",
                self.state,
                self.num_pais_rem,
                c_max,
                c
            );
            return;
        }
        if self.num_pais_rem == 0 {
            let penalty = num_meld + num_dazi + exist_eye - 5;
            let num_fuuros = (14 - self.num_tehai) / 3;
            let cur_s = 9 - 2 * num_meld - num_dazi - 2 * exist_eye - num_fuuros + penalty;
            *shanten = std::cmp::min(*shanten, cur_s);
            *c_max = std::cmp::max(*c_max, c);
            log_if!(
                self.verbose,
                "search end. cur state: {:?}. cause: c_rem == 0; => s: {}, c_max: {}",
                self.state,
                shanten,
                c_max,
            );
            return;
        }
        if let Some(dazi) = self.try_take_2(i) {
            log_if!(self.verbose, "take {:?} as dazi begin", dazi);
            self.search_by_take_2(i, shanten, c_max, k, exist_eye, num_meld, num_dazi + 1);
            self.rollback_pais(&dazi);
            log_if!(
                self.verbose,
                "take {:?} as dazi done, s: {}",
                dazi,
                *shanten
            );
        }

        for take_idx in 0..self.num_pais_rem {
            if let Some(pai) = self.try_take_1(take_idx) {
                self.search_by_take_2(i + 1, shanten, c_max, k, exist_eye, num_meld, num_dazi);
                self.rollback_pais(&[pai]);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use convlog::mjai::*;
    use convlog::pai::get_pais_from_str;

    use super::*;

    #[test]
    fn test_chi_to_pais() {
        let f = Fuuro::Chi {
            target: 0,
            pai: Pai::Man1,
            consumed: Consumed2::from([Pai::Man2, Pai::Man3]),
        };
        let pais = f.into_pais();
        assert_eq!(pais.len(), 3);
        assert_eq!(pais[0], Pai::Man1);
        assert_eq!(pais[1], Pai::Man2);
        assert_eq!(pais[2], Pai::Man3);
    }
    #[test]
    fn test_pon_to_pais() {
        let f = Fuuro::Pon {
            target: 0,
            pai: Pai::Man1,
            consumed: Consumed2::from([Pai::Man1, Pai::Man1]),
        };
        let pais = f.into_pais();
        assert_eq!(pais.len(), 3);
        assert_eq!(pais[0], Pai::Man1);
        assert_eq!(pais[1], Pai::Man1);
        assert_eq!(pais[2], Pai::Man1);
    }
    #[test]
    fn test_kan_to_pais() {
        let cases = [
            Fuuro::Daiminkan {
                target: 0,
                pai: Pai::Man1,
                consumed: Consumed3::from([Pai::Man1, Pai::Man1, Pai::Man1]),
            },
            Fuuro::Kakan {
                pai: Pai::Man1,
                previous_pon_target: 0,
                previous_pon_pai: Pai::Man1,
                consumed: Consumed2::from([Pai::Man1, Pai::Man1]),
            },
            Fuuro::Ankan {
                consumed: Consumed4::from([Pai::Man1, Pai::Man1, Pai::Man1, Pai::Man1]),
            },
        ];
        for case in cases {
            let pais = case.into_pais();
            assert_eq!(pais.len(), 4);
            assert_eq!(pais[0], Pai::Man1);
            assert_eq!(pais[1], Pai::Man1);
            assert_eq!(pais[2], Pai::Man1);
            assert_eq!(pais[3], Pai::Man1);
        }
    }

    enum Case {
        Normal { i: &'static str, s: i32 },
        Chiitoi { i: &'static str, s: i32 },
        Kokushi { i: &'static str, s: i32 },
    }
    #[test]
    fn test_iter_stat_pais() {
        let case: Vec<Case> = Vec::<Case>::from([
            // Case::Normal {
            //     i: "0m12356p4699s4m222z",
            //     s: 1,
            // },
            // Case::Normal {
            //     i: "0m12356p4699s4m",
            //     s: 1,
            // },
            // Case::Normal {
            //     i: "123456789p123s55m",
            //     s: -1,
            // },
            // Case::Normal {
            //     i: "12345678p123s55m1z",
            //     s: 0,
            // },
            // Case::Normal {
            //     i: "12345678p12s55m12z",
            //     s: 1,
            // },
            // Case::Normal {
            //     i: "0m125p1469s24z6p",
            //     s: 3,
            // },
            // Case::Normal {
            //     i: "0m1256p469s24z9s",
            //     s: 2,
            // },
            // Case::Normal {
            //     i: "0m1256p4699s4z3p",
            //     s: 1,
            // },
            // Case::Normal {
            //     i: "245m12356p99s222z4p",
            //     s: 0,
            // },
            // Case::Normal {
            //     i: "45m123456p99s222z2m",
            //     s: 0,
            // },
            // Case::Normal {
            //     i: "45m123456p99s222z3m",
            //     s: -1,
            // },
            // Case::Normal {
            //     i: "45m235678p399s22z6s",
            //     s: 2,
            // },
            // Case::Normal {
            //     i: "45m23568p3699s22z4m",
            //     s: 3,
            // },
            // Case::Normal {
            //     i: "445m2358p23469s2z9m",
            //     s: 4,
            // },
            // Case::Normal {
            //     i: "49m2358p23469s24z1m",
            //     s: 5,
            // },
            // Case::Normal {
            //     i: "149m258p2369s124z7s",
            //     s: 6,
            // },
            Case::Kokushi {
                i: "159m19p19s1234677z",
                s: 0,
            },
            Case::Kokushi {
                i: "159m19p19s1236677z",
                s: 1,
            },
            Case::Chiitoi {
                i: "458m666p116688s55z",
                s: 1, // normal 2
            },
            Case::Chiitoi {
                i: "44m6666p116688s55z",
                s: 1, // normal 2
            },
            Case::Chiitoi {
                i: "4444m6666p1111s55z",
                s: 5, 
            },
            Case::Normal {
                i: "4444m6666p1111s55z",
                s: 1, 
            },
        ]);
        for c in case {
            match c {
                Case::Normal { i: input, .. }
                | Case::Kokushi { i: input, .. }
                | Case::Chiitoi { i: input, .. } => {
                    println!("input: '{}'", input);
                    let mut helper =
                        ShantenHelper::new(&Tehai::from(get_pais_from_str(input).unwrap()));
                    match c {
                        Case::Normal { i: input, s } => {
                            let normal = helper.get_normal_shanten();
                            println!("shanten: {} for '{}'(normal)", s, input);
                            assert_eq!(s, normal, "for '{}'", input);
                        }
                        Case::Kokushi { i: input, s } => {
                            let kokushi = helper.get_kokushi_shanten();
                            println!("shanten: {} for '{}'(kokushi)", s, input);
                            assert_eq!(s, kokushi, "for '{}'", input);
                        }
                        Case::Chiitoi { i: input, s } => {
                            let chiitoi = helper.get_chiitoi_shanten();
                            println!("shanten: {} for '{}'(chiitoi)", s, input);
                            assert_eq!(s, chiitoi, "for '{}'", input);
                        }
                    }
                }
            }
        }
    }
}
