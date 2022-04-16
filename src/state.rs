use crate::tehai::Tehai;
use crate::{log, log_if};

use anyhow::anyhow;
use anyhow::{Context, Result};
use convlog::mjai::{Consumed2, Consumed3, Consumed4, Event};
use convlog::Pai;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::convert::TryFrom;

#[derive(Debug, Clone, Default, Serialize)]
pub struct State {
    #[serde(skip)]
    actor: u8,

    pub tehai: Tehai,
    pub fuuros: Vec<Fuuro>,
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

const PAIS_VEC_LEN: usize = 48;

struct ShantenHelper {
    pais: [i32; PAIS_VEC_LEN],
    num_pais_rem: i32,
    num_tehai: i32,

    verbose: bool,
    state: Vec<Vec<Pai>>,
}

impl ShantenHelper {
    fn new(tehai: &Tehai) -> Self {
        // collect all pais in tehai and fuuros
        let mut pais: [i32; 48] = [0i32; 48];
        let mut num_pais = 0;
        for pai in tehai.view().iter() {
            // Pai.as_unify_u8 has handled 0m 0p 0s => 5m 5p 5s
            num_pais += 1;
            pais[pai.as_unify_u8() as usize] += 1;
        }
        // assert!(num_pais == 14 || num_pais == 13);
        Self {
            pais,
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
        self.pais.iter().enumerate().for_each(|(idx, num)| {
            if *num == 0 {
                return;
            }
            if let Ok(pai) = Pai::try_from(idx as u8) {
                num_kind += match pai {
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
                        if *num > 1 {
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
        // 6 at max for chiitoi
        let mut shanten = 6i32;
        // there is any fuuro, then we can not get chiitoi
        if self.num_tehai < 13 {
            return shanten;
        }
        let mut num_kind = 0;
        self.pais.iter().enumerate().for_each(|(idx, num)| {
            if *num == 0 {
                return;
            }
            if let Ok(_pai) = Pai::try_from(idx as u8) {
                if *num >= 2 {
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
        let num_fuuros = (14 - self.num_tehai) / 3;
        let mut shanten = 8i32;
        let mut c_max = 0i32;
        let k = (self.num_pais_rem - 2) / 3;
        for eye in ShantenHelper::eyes(&self.pais) {
            // try to get the shanten with this eye
            let eye_array = [eye, eye];
            log_if!(self.verbose, "take {:?} as eye begin", eye_array);
            self.take(&eye_array);
            self.search_by_take_3(0, 11, &mut shanten, &mut c_max, k, 1, num_fuuros);
            self.rollback_pais(&eye_array);
            log_if!(
                self.verbose,
                "take {:?} as eye done, s: {}",
                eye_array,
                shanten
            );
        }
        // try to get the shanten without eye
        log_if!(self.verbose, "take nothing as eye begin");
        self.search_by_take_3(0, 11, &mut shanten, &mut c_max, k, 0, num_fuuros);
        log_if!(self.verbose, "take nothing as eye done, s: {}", shanten);
        shanten
    }

    fn eyes(pais: &[i32; 48]) -> Vec<Pai> {
        let mut eyes = Vec::new();
        for (idx, num) in pais.iter().enumerate() {
            if *num < 2 {
                continue;
            }
            if let Ok(pai) = Pai::try_from(idx as u8) {
                eyes.push(pai);
            }
        }
        eyes
    }

    fn take<const LEN: usize>(&mut self, pais: &[Pai; LEN]) {
        for idx in 0..pais.len() {
            self.pais[pais[idx].as_unify_u8() as usize] -= 1;
        }
        self.num_pais_rem -= pais.len() as i32;
        self.state.push(pais.to_vec());
    }

    fn rollback_pais<const LEN: usize>(&mut self, pais: &[Pai; LEN]) {
        for p in pais {
            self.pais[p.as_unify_u8() as usize] += 1;
        }
        self.num_pais_rem += pais.len() as i32;
        self.state.pop();
    }

    fn next_not_zero(&self, take_idx: usize) -> usize {
        for idx in take_idx..PAIS_VEC_LEN {
            if self.pais[idx] > 0 {
                return idx;
            }
        }
        PAIS_VEC_LEN
    }

    fn try_take_3(&mut self, take_idx: usize) -> Option<([Pai; 3], usize)> {
        for idx in take_idx..48 {
            let idx = idx;
            let num = self.pais[idx];
            if num < 1 {
                continue;
            }
            // 1~9s, 1~9p, 1~9m
            if (11 <= idx && idx <= 19) || (21 <= idx && idx <= 29) || (31 <= idx && idx <= 39) {
                // try to get triplet (AAA)
                if num >= 3 {
                    if let Ok(pai) = Pai::try_from(idx as u8) {
                        let meld = [pai, pai, pai];
                        self.take(&meld);
                        return Some((meld, idx));
                    }
                }
                // 1~7s, 1~7p, 1~7m
                if (11 <= idx && idx < 18) || (21 <= idx && idx < 28) || (31 <= idx && idx < 38) {
                    // try to get sequence (ABC)
                    if self.pais[idx + 1] < 1 || self.pais[idx + 2] < 1 {
                        continue;
                    }
                    let meld = [
                        Pai::try_from(idx as u8).unwrap(),
                        Pai::try_from((idx + 1) as u8).unwrap(),
                        Pai::try_from((idx + 2) as u8).unwrap(),
                    ];
                    self.take(&meld);
                    return Some((meld, idx));
                }
            }
            // try to get triplet (AAA)
            if (41 <= idx && idx <= 47) && num >= 3 {
                if let Ok(pai) = Pai::try_from(idx as u8) {
                    let meld = [pai, pai, pai];
                    self.take(&meld);
                    return Some((meld, idx));
                }
            }
        }
        None
    }

    fn try_take_2(&mut self, take_idx: usize) -> Option<([Pai; 2], usize)> {
        for idx in take_idx..48 {
            if self.pais[idx] < 1 {
                continue;
            }
            // 1~9s, 1~9p, 1~9m
            if (11 <= idx && idx <= 19) || (21 <= idx && idx <= 29) || (31 <= idx && idx <= 39) {
                // try get pair
                if self.pais[idx] > 1 {
                    if let Ok(pai) = Pai::try_from(idx as u8) {
                        let res = [pai, pai];
                        self.take(&res);
                        return Some((res, idx));
                    }
                }
                if idx % 10 > 8 {
                    continue;
                }
                // 1~8s, 1~8p, 1~8m
                if self.pais[idx + 1] > 0 {
                    // PENCHAN/RYANMEN
                    let res = [
                        Pai::try_from(idx as u8).unwrap(),
                        Pai::try_from((idx + 1) as u8).unwrap(),
                    ];
                    self.take(&res);
                    return Some((res, idx));
                } else if idx % 10 < 8 && self.pais[idx + 2] > 0 {
                    // 1~7s, 1~7p, 1~7m, KANCHAN
                    let res = [
                        Pai::try_from(idx as u8).unwrap(),
                        Pai::try_from((idx + 2) as u8).unwrap(),
                    ];
                    self.take(&res);
                    return Some((res, idx));
                }
            }
            // try get pair
            if 41 <= idx && idx <= 47 && self.pais[idx] > 1 {
                if let Ok(pai) = Pai::try_from(idx as u8) {
                    let res = [pai, pai];
                    self.take(&res);
                    return Some((res, idx));
                }
            }
        }
        None
    }

    fn try_take_1(&mut self, _take_idx: usize) -> Option<Pai> {
        for idx in 0..PAIS_VEC_LEN {
            let num = self.pais[idx];
            if num < 1 {
                continue;
            }
            if let Ok(pai) = Pai::try_from(idx as u8) {
                let res = [pai];
                self.take(&res);
                return Some(pai);
            }
        }
        None
    }

    fn search_by_take_3(
        &mut self,
        level: i32,
        begin_idx: usize,
        shanten: &mut i32,
        c_max: &mut i32,
        k: i32,
        exist_eye: i32,
        num_meld: i32,
    ) {
        log_if!(
            self.verbose,
            "enter search_by_3 with i: {}, c_rem: {}, s: {}, c_max: {}",
            begin_idx,
            self.num_pais_rem,
            shanten,
            c_max
        );
        if begin_idx >= PAIS_VEC_LEN || level > 3 {
            self.search_by_take_2(11, shanten, c_max, k, exist_eye, num_meld, 0);
            return;
        }

        // take a meld TODO: handle AAABC
        if let Some((meld, next_search_idx)) = self.try_take_3(begin_idx) {
            log_if!(self.verbose, "take {:?} as meld begin", meld);
            self.search_by_take_3(
                level + 1,
                next_search_idx,
                shanten,
                c_max,
                k,
                exist_eye,
                num_meld + 1,
            );
            self.rollback_pais(&meld);
            log_if!(
                self.verbose,
                "take {:?} as meld done, s: {}",
                meld,
                *shanten
            );
        }
        log_if!(
            self.verbose,
            "take nothing as meld begin, idx: {}",
            begin_idx
        );
        let next_search_idx = self.next_not_zero(begin_idx + 1);
        self.search_by_take_3(
            level,
            next_search_idx,
            shanten,
            c_max,
            k,
            exist_eye,
            num_meld,
        );
        log_if!(
            self.verbose,
            "take nothing as meld done, idx: {}, s: {}",
            begin_idx,
            *shanten
        );
    }

    fn search_by_take_2(
        &mut self,
        begin_idx: usize,
        shanten: &mut i32,
        c_max: &mut i32,
        k: i32,
        exist_eye: i32,
        num_meld: i32,
        num_dazi: i32,
    ) {
        log_if!(
            self.verbose,
            "enter search_by_2 with i: {}, c_rem: {}, s: {}, c_max: {}, g: {}, gp: {}",
            begin_idx,
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
            let penalty = std::cmp::max(num_meld + num_dazi + exist_eye - 5, 0);
            let q = if num_meld + num_dazi + exist_eye <= 4 {
                1
            } else {
                exist_eye
            };
            let cur_s = 9 - 2 * num_meld - (num_dazi + exist_eye) - q + penalty;
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
        if let Some((dazi, next_search_idx)) = self.try_take_2(begin_idx) {
            log_if!(self.verbose, "take {:?} as dazi begin", dazi);
            self.search_by_take_2(
                next_search_idx,
                shanten,
                c_max,
                k,
                exist_eye,
                num_meld,
                num_dazi + 1,
            );
            self.rollback_pais(&dazi);
            log_if!(
                self.verbose,
                "take {:?} as dazi done, s: {}",
                dazi,
                *shanten
            );
        }

        let mut stack = Vec::with_capacity(self.num_pais_rem as usize);
        // take all remaining pai and update the shanten number
        for _idx in 0..self.num_pais_rem {
            if let Some(pai) = self.try_take_1(0) {
                stack.push(pai);
                log_if!(self.verbose, "take [{:?}] as pai begin", pai);
            }
        }
        self.search_by_take_2(0, shanten, c_max, k, exist_eye, num_meld, num_dazi);
        // rollbacks
        for pai in stack.iter().rev() {
            self.rollback_pais(&[*pai]);
            log_if!(self.verbose, "take [{:?}] as pai end", pai);
        }
    }
}

#[cfg(test)]
mod tests {

    use convlog::mjai::*;
    use convlog::pai::get_pais_from_str;

    use super::*;

    #[test]
    fn test_normal_shanten() {
        let cases: Vec<(&'static str, i32)> = Vec::from([
            ("1112234567899s4s", 0),
            ("11112345678999s", -1),
            ("11122345678999s", -1),
            ("11123345678999s", -1),
            ("11123445678999s", -1),
            ("11123455678999s", -1),
            ("11123456678999s", -1),
            ("11123456778999s", -1),
            ("11123456788999s", -1),
            ("11123456789999s", -1),
            ("40m12356p4699s222z", 1),
            ("0m12356p4699s4m", 1),
            ("123456789p123s55m", -1),
            ("12345678p123s55m1z", 0),
            ("12345678p12s55m12z", 1),
            ("0m125p1469s24z6p", 3),
            ("0m1256p469s24z9s", 2),
            ("0m1256p4699s4z3p", 1),
            ("245m12356p99s222z4p", 0),
            ("45m123456p99s222z2m", 0),
            ("45m123456p99s222z3m", -1),
            ("45m235678p399s22z6s", 2),
            ("45m23568p3699s22z4m", 3),
            ("445m2358p23469s2z9m", 4),
            ("49m2358p23469s24z1m", 5),
            ("149m258p2369s124z7s", 6),
            ("4444m6666p1111s55z", 1),
        ]);
        for (input, s) in cases {
            println!("input: '{}'", input);
            let mut helper = ShantenHelper::new(&Tehai::from(get_pais_from_str(input).unwrap()));
            let normal = helper.get_normal_shanten();
            println!("shanten: {} for '{}'(normal)", s, input);
            assert_eq!(s, normal, "for '{}'", input);
        }
    }

    #[test]
    fn test_chiitoi_shanten() {
        let cases = Vec::from([
            ("458m666p116688s55z", 1), // normal 2
            ("44m6666p116688s55z", 1), // normal 2
            ("4444m6666p1111s55z", 5),
        ]);
        for (input, s) in cases {
            println!("input: '{}'", input);
            let helper = ShantenHelper::new(&Tehai::from(get_pais_from_str(input).unwrap()));
            let normal = helper.get_chiitoi_shanten();
            println!("shanten: {} for '{}'(normal)", s, input);
            assert_eq!(s, normal, "for '{}'", input);
        }
    }

    #[test]
    fn test_kokushi_shanten() {
        let cases = Vec::from([
            //
            ("159m19p19s1234677z", 0),
            ("159m19p19s1236677z", 1),
        ]);
        for (input, s) in cases {
            println!("input: '{}'", input);
            let helper = ShantenHelper::new(&Tehai::from(get_pais_from_str(input).unwrap()));
            let normal = helper.get_kokushi_shanten();
            println!("shanten: {} for '{}'(normal)", s, input);
            assert_eq!(s, normal, "for '{}'", input);
        }
    }
}
