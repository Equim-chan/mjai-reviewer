use super::mjai;
use super::tenhou;
use super::Pai;
use std::str;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("invalid naki string: {0:?}")]
    InvalidNaki(String),

    #[error("invalid pai string")]
    InvalidPai(#[source] <u8 as std::str::FromStr>::Err),

    #[error("insufficient dora indicators: at kyoku {kyoku} honba {honba}")]
    InsufficientDoraIndicators { kyoku: u8, honba: u8 },

    #[error("insufficient takes sequence size: at kyoku {kyoku} honba {honba} for actor {actor}")]
    InsufficientTakes { kyoku: u8, honba: u8, actor: u8 },

    #[error(
        "insufficient discards sequence size: at kyoku {kyoku} honba {honba} for actor {actor}"
    )]
    InsufficientDiscards { kyoku: u8, honba: u8, actor: u8 },
}

pub type Result<T> = std::result::Result<T, ConvertError>;

/// Transform a tenhou.net/6 format log into mjai format.
pub fn tenhou_to_mjai(log: &tenhou::Log) -> Result<Vec<mjai::Event>> {
    let mut events = vec![];

    events.push(mjai::Event::StartGame {
        kyoku_first: log.game_length as u8,
        aka_flag: log.has_aka,
        names: log.names.clone(),
    });

    for kyoku in &log.kyokus {
        tenhou_kyoku_to_mjai_events(&mut events, kyoku)?;
    }

    events.push(mjai::Event::EndGame);

    Ok(events)
}

fn tenhou_kyoku_to_mjai_events(events: &mut Vec<mjai::Event>, kyoku: &tenhou::Kyoku) -> Result<()> {
    // first of all, transform all takes and discards to events.
    let mut take_events: Vec<_> = (0..4)
        .map(|i| take_action_to_events(i, &kyoku.action_tables[usize::from(i)].takes))
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(|v| v.into_iter().peekable())
        .collect();
    let mut discard_events: Vec<_> = (0..4)
        .map(|i| discard_action_to_events(i, &kyoku.action_tables[usize::from(i)].discards))
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(|v| v.into_iter().peekable())
        .collect();

    // then emit the events in order.
    let oya = kyoku.meta.kyoku_num % 4;
    let mut dora_feed = kyoku.dora_indicators.clone().into_iter();
    let mut reach_flag: Option<usize> = None;
    let mut last_tsumo = Pai(0);
    let mut need_new_dora = false;

    events.push(mjai::Event::StartKyoku {
        bakaze: Pai(41 + kyoku.meta.kyoku_num / 4),
        kyoku: kyoku.meta.kyoku_num % 4 + 1,
        honba: kyoku.meta.honba,
        kyotaku: kyoku.meta.kyotaku,
        dora_marker: dora_feed
            .next()
            .ok_or(ConvertError::InsufficientDoraIndicators {
                kyoku: kyoku.meta.kyoku_num,
                honba: kyoku.meta.honba,
            })?,
        oya,
        scores: kyoku.scoreboard,
        tehais: [
            kyoku.action_tables[0].haipai,
            kyoku.action_tables[1].haipai,
            kyoku.action_tables[2].haipai,
            kyoku.action_tables[3].haipai,
        ],
    });

    let mut actor = usize::from(oya);
    loop {
        let take = take_events[actor]
            .next()
            .ok_or(ConvertError::InsufficientTakes {
                kyoku: kyoku.meta.kyoku_num,
                honba: kyoku.meta.honba,
                actor: actor as u8,
            })?;

        // record the pai so that it can be filled in tsumogiri dahai.
        if let mjai::Event::Tsumo { pai, .. } = take {
            last_tsumo = pai;
        }

        // if a reach event was emitted before, set it as accepted now.
        if let Some(actor) = reach_flag.take() {
            events.push(mjai::Event::ReachAccepted { actor: actor as u8 });
        }

        // skip one discard if it is daiminkan.
        // and then immediately consume the next take event from the same actor.
        if let mjai::Event::Daiminkan { .. } = take {
            need_new_dora = true;
            discard_events[actor].next(); // cannot use .skip(1) here as the types do not match
            continue;
        }

        // emit the take event
        events.push(take);

        // check if the kyoku ends here, can be ryukyoku (九種九牌) or tsumo.
        // here it simply checks if there is no more discard for current actor.
        if discard_events[actor].peek().is_none() {
            match kyoku.end_status {
                tenhou::kyoku::EndStatus::Hora => {
                    events.push(mjai::Event::Hora {
                        actor: actor as u8,
                        target: actor as u8,
                    });
                }
                tenhou::kyoku::EndStatus::Ryukyoku => {
                    events.push(mjai::Event::Ryukyoku);
                }
            }

            events.push(mjai::Event::EndKyoku);
            break;
        }

        let discard = discard_events[actor]
            .next()
            .ok_or(ConvertError::InsufficientDiscards {
                kyoku: kyoku.meta.kyoku_num,
                honba: kyoku.meta.honba,
                actor: actor as u8,
            })?
            .fill_possible_tsumogiri(last_tsumo);
        events.push(discard.clone());

        // process previous minkan.
        if need_new_dora {
            events.push(mjai::Event::Dora {
                dora_marker: dora_feed
                    .next()
                    .ok_or(ConvertError::InsufficientDoraIndicators {
                        kyoku: kyoku.meta.kyoku_num,
                        honba: kyoku.meta.honba,
                    })?,
            });
            need_new_dora = false;
        }

        // process reach declare.
        // a reach declare actually consists of two events (reach + dahai).
        if let mjai::Event::Reach { .. } = discard {
            reach_flag = Some(actor);

            let dahai = discard_events[actor]
                .next()
                .ok_or(ConvertError::InsufficientDiscards {
                    kyoku: kyoku.meta.kyoku_num,
                    honba: kyoku.meta.honba,
                    actor: actor as u8,
                })?
                .fill_possible_tsumogiri(last_tsumo);
            events.push(dahai);
        }

        // check if the kyoku ends here, can be ryukyoku or ron.
        // here it simply checks if there is no more take for every single actor.
        if (0..4).all(|i| take_events[i].peek().is_none()) {
            match kyoku.end_status {
                tenhou::kyoku::EndStatus::Hora => {
                    for hora in &kyoku.hora_status {
                        events.push(mjai::Event::Hora {
                            actor: hora.who,
                            target: hora.target,
                        });
                    }
                }
                tenhou::kyoku::EndStatus::Ryukyoku => {
                    events.push(mjai::Event::Ryukyoku);
                }
            }

            events.push(mjai::Event::EndKyoku);
            break;
        }

        // check if the last discard was ankan or kakan.
        // for kan, it will immediately consume the next take event from the same actor.
        match discard {
            mjai::Event::Ankan { .. } => {
                // ankan triggers a dora event immediately.
                events.push(mjai::Event::Dora {
                    dora_marker: dora_feed.next().ok_or(
                        ConvertError::InsufficientDoraIndicators {
                            kyoku: kyoku.meta.kyoku_num,
                            honba: kyoku.meta.honba,
                        },
                    )?,
                });
                continue;
            }
            mjai::Event::Kakan { .. } => {
                need_new_dora = true;
                continue;
            }
            _ => (),
        }

        // decide who is the next actor.
        // if someone takes taki of the previous discard, then it will be him,
        // otherwise it will be the shimocha.
        actor = (0..4)
            .filter(|&i| i != actor)
            .find(|&i| {
                if let Some(take) = take_events[i].peek() {
                    if let Some(target) = take.naki_target() {
                        return target == actor as u8;
                    }
                }

                false
            })
            .unwrap_or((actor + 1) % 4);
    }

    Ok(())
}

fn take_action_to_events(actor: u8, takes: &[tenhou::ActionItem]) -> Result<Vec<mjai::Event>> {
    takes
        .iter()
        .map(|take| match take {
            tenhou::ActionItem::Pai(pai) => Ok(mjai::Event::Tsumo { actor, pai: *pai }),
            tenhou::ActionItem::Naki(naki_string) => {
                let naki = naki_string.as_bytes();

                if naki.contains(&b'c') {
                    // chi
                    // you can only chi from kamicha right...?

                    if naki_string.len() != 7 {
                        return Err(ConvertError::InvalidNaki(naki_string.clone()));
                    }

                    // e.g. "c275226" => chi 7p with 06p from kamicha
                    Ok(mjai::Event::Chi {
                        actor,
                        target: (actor + 3) % 4,
                        pai: pai_from_bytes(&naki[1..3])?,
                        consumed: [pai_from_bytes(&naki[3..5])?, pai_from_bytes(&naki[5..7])?],
                    })
                } else if let Some(idx) = naki_string.find('p') {
                    // pon

                    if naki_string.len() != 7 {
                        return Err(ConvertError::InvalidNaki(naki_string.clone()));
                    }

                    match idx {
                        // from kamicha
                        // e.g. "p252525" => pon 5p from kamicha
                        0 => Ok(mjai::Event::Pon {
                            actor,
                            target: (actor + 3) % 4,
                            pai: pai_from_bytes(&naki[1..3])?,
                            consumed: [pai_from_bytes(&naki[3..5])?, pai_from_bytes(&naki[5..7])?],
                        }),

                        // from toimen
                        // e.g. "12p1212" => pon 2m from toimen
                        2 => Ok(mjai::Event::Pon {
                            actor,
                            target: (actor + 2) % 4,
                            pai: pai_from_bytes(&naki[3..5])?,
                            consumed: [pai_from_bytes(&naki[0..2])?, pai_from_bytes(&naki[5..7])?],
                        }),
                        // from shimocha
                        // e.g. "3737p37" => pon 7s from shimocha
                        4 => Ok(mjai::Event::Pon {
                            actor,
                            target: (actor + 1) % 4,
                            pai: pai_from_bytes(&naki[5..7])?,
                            consumed: [pai_from_bytes(&naki[0..2])?, pai_from_bytes(&naki[2..4])?],
                        }),

                        // ???
                        _ => Err(ConvertError::InvalidNaki(naki_string.clone())),
                    }
                } else if let Some(idx) = naki_string.find('m') {
                    // daiminkan

                    if naki_string.len() != 9 {
                        return Err(ConvertError::InvalidNaki(naki_string.clone()));
                    }

                    match idx {
                        // from kamicha
                        // e.g. "m39393939" => kan 9s from kamicha
                        0 => Ok(mjai::Event::Daiminkan {
                            actor,
                            target: (actor + 3) % 4,
                            pai: pai_from_bytes(&naki[1..3])?,
                            consumed: [
                                pai_from_bytes(&naki[3..5])?,
                                pai_from_bytes(&naki[5..7])?,
                                pai_from_bytes(&naki[7..9])?,
                            ],
                        }),

                        // from toimen
                        // e.g. "26m262626" => kan 6p from toimen
                        2 => Ok(mjai::Event::Daiminkan {
                            actor,
                            target: (actor + 2) % 4,
                            pai: pai_from_bytes(&naki[3..5])?,
                            consumed: [
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[5..7])?,
                                pai_from_bytes(&naki[7..9])?,
                            ],
                        }),

                        // from shimocha
                        // e.g. "131313m13" => kan 3m from shimocha
                        6 => Ok(mjai::Event::Daiminkan {
                            actor,
                            target: (actor + 1) % 4,
                            pai: pai_from_bytes(&naki[7..9])?,
                            consumed: [
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[2..4])?,
                                pai_from_bytes(&naki[4..6])?,
                            ],
                        }),

                        // ???
                        _ => Err(ConvertError::InvalidNaki(naki_string.clone())),
                    }
                } else {
                    Err(ConvertError::InvalidNaki(naki_string.clone()))
                }
            }
        })
        .collect()
}

fn discard_action_to_events(
    actor: u8,
    discards: &[tenhou::ActionItem],
) -> Result<Vec<mjai::Event>> {
    let mut ret = vec![];

    for discard in discards {
        match discard {
            tenhou::ActionItem::Pai(pai) => {
                let ev = mjai::Event::Dahai {
                    actor,
                    pai: *pai, // must be filled later if it is tsumogiri
                    tsumogiri: pai.0 == 60,
                };

                ret.push(ev);
            }

            tenhou::ActionItem::Naki(naki_string) => {
                let naki = naki_string.as_bytes();

                // only ankan, kakan and reach are possible
                if let Some(idx) = naki_string.find('k') {
                    // kakan

                    if naki_string.len() != 9 {
                        return Err(ConvertError::InvalidNaki(naki_string.clone()));
                    }

                    let ev = match idx {
                        // previously pon from toimen
                        // e.g. "k16161616" => pon 6m from kamicha then kan
                        0 => mjai::Event::Kakan {
                            actor,
                            pai: pai_from_bytes(&naki[1..3])?,
                            consumed: [
                                pai_from_bytes(&naki[3..5])?,
                                pai_from_bytes(&naki[5..7])?,
                                pai_from_bytes(&naki[7..9])?,
                            ],
                        },

                        // previously pon from toimen
                        // e.g. "41k414141" => pon 1z from toimen then kan
                        2 => mjai::Event::Kakan {
                            actor,
                            pai: pai_from_bytes(&naki[3..5])?,
                            consumed: [
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[5..7])?,
                                pai_from_bytes(&naki[7..9])?,
                            ],
                        },

                        // previously pon from shimocha
                        // e.g. "4646k4646" => pon 6z from shimocha then kan
                        4 => mjai::Event::Kakan {
                            actor,
                            pai: pai_from_bytes(&naki[5..7])?,
                            consumed: [
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[2..4])?,
                                pai_from_bytes(&naki[7..9])?,
                            ],
                        },

                        // ???
                        _ => {
                            return Err(ConvertError::InvalidNaki(naki_string.clone()));
                        }
                    };

                    ret.push(ev);
                } else if naki.contains(&b'a') {
                    // ankan
                    // for ankan, 'a' can only appear at [6]
                    // e.g. "424242a42" => ankan 2z

                    if naki_string.len() != 9 {
                        return Err(ConvertError::InvalidNaki(naki_string.clone()));
                    }

                    let pai = pai_from_bytes(&naki[7..9])?;
                    let ev = mjai::Event::Ankan {
                        actor,
                        consumed: [
                            pai_from_bytes(&naki[0..2])?,
                            pai_from_bytes(&naki[2..4])?,
                            pai_from_bytes(&naki[4..6])?,
                            pai,
                        ],
                    };

                    ret.push(ev);
                } else {
                    // reach
                    // e.g. "r35" => discard 5s to reach

                    if naki_string.len() != 3 {
                        return Err(ConvertError::InvalidNaki(naki_string.clone()));
                    }

                    let pai = pai_from_bytes(&naki[1..3])?;

                    ret.push(mjai::Event::Reach { actor });
                    ret.push(mjai::Event::Dahai {
                        actor,
                        pai, // must be filled later if it is tsumogiri
                        tsumogiri: pai.0 == 60,
                    });
                }
            }
        };
    }

    Ok(ret)
}

#[inline]
fn pai_from_bytes(b: &[u8]) -> Result<Pai> {
    let s = unsafe { str::from_utf8_unchecked(b) };
    let pai = Pai(s.parse().map_err(ConvertError::InvalidPai)?);

    Ok(pai)
}

impl mjai::Event {
    #[inline]
    fn fill_possible_tsumogiri(self, last_tsumo: Pai) -> Self {
        match self {
            mjai::Event::Dahai {
                actor,
                tsumogiri: true,
                ..
            } => mjai::Event::Dahai {
                actor,
                pai: last_tsumo,
                tsumogiri: true,
            },
            _ => self,
        }
    }
}
