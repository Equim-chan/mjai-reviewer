use crate::tehai::Tehai;

use anyhow::anyhow;
use anyhow::{Context, Result};
use convlog::mjai::{Consumed2, Consumed3, Consumed4, Event};
use convlog::Pai;
use serde::Serialize;

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

    /// Argument `event` must be either a
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
                self.tehai.remove_multiple(&consumed.0);

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
                self.tehai.remove_multiple(&consumed.0);

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
                self.tehai.remove_multiple(&consumed.0);

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
                        } if Consumed3([pon_pai, pon_consumed.0[0], pon_consumed.0[1]])
                            == consumed =>
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
                self.tehai.remove_multiple(&consumed.0);

                let fuuro = Fuuro::Ankan { consumed };
                self.fuuros.push(fuuro);
            }

            _ => (),
        };

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Fuuro {
    Chi {
        target: u8,
        pai: Pai,
        consumed: Consumed2,
    },
    Pon {
        target: u8,
        pai: Pai,
        consumed: Consumed2,
    },
    Daiminkan {
        target: u8,
        pai: Pai,
        consumed: Consumed3,
    },
    Kakan {
        pai: Pai,
        previous_pon_target: u8,
        previous_pon_pai: Pai,
        consumed: Consumed2,
    },
    Ankan {
        consumed: Consumed4,
    },
}
