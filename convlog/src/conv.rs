use crate::mjai;
use crate::tenhou;
use crate::Pai;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::TryFrom;

use thiserror::Error;

const BACKTRACK_LIMIT: u8 = 10;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("invalid naki string: {0:?}")]
    InvalidNaki(String),

    #[error("invalid pai string: {0:?}")]
    InvalidPai(String),

    #[error("insufficient dora indicators: at kyoku={kyoku} honba={honba}")]
    InsufficientDoraIndicators { kyoku: u8, honba: u8 },

    #[error(
        "insufficient take sequence size: \
        at kyoku={kyoku} honba={honba} for actor={actor}"
    )]
    InsufficientTakes { kyoku: u8, honba: u8, actor: u8 },

    #[error(
        "insufficient discard sequence size: \
        at kyoku={kyoku} honba={honba} for actor={actor}"
    )]
    InsufficientDiscards { kyoku: u8, honba: u8, actor: u8 },

    #[error("tsumogiri should not exist in discard table")]
    UnexpectedTsumogiri,

    #[error(
        "unexpected naki: \
        at kyoku={kyoku} honba={honba} for actor={actor}: \
        action={action:?}, expected pai={last_dahai:?} \
        from={last_actor:?}"
    )]
    UnexpectedNaki {
        action: mjai::Event,
        last_dahai: Pai,
        last_actor: Option<u8>,
        kyoku: u8,
        honba: u8,
        actor: u8,
    },
}

pub type Result<T> = std::result::Result<T, ConvertError>;

#[derive(Debug)]
struct BackTrack {
    use_the_first_branch: bool,
}

/// Transform a tenhou.net/6 format log into mjai format.
pub fn tenhou_to_mjai(log: &tenhou::Log) -> Result<Vec<mjai::Event>> {
    let mut events = vec![mjai::Event::StartGame {
        kyoku_first: log.game_length as u8,
        aka_flag: log.has_aka,
        names: log.names.clone(),
    }];

    let mut backtracks = HashMap::new();
    for kyoku in &log.kyokus {
        backtracks.clear();
        let mut first_error = None;

        let result = (0..BACKTRACK_LIMIT).find_map(|_| {
            match tenhou_kyoku_to_mjai_events(kyoku, &mut backtracks) {
                Ok(kyoku_events) => Some(kyoku_events),
                Err(err) => {
                    first_error.get_or_insert(err);
                    None
                }
            }
        });

        if let Some(kyoku_events) = result {
            events.extend(kyoku_events);
        } else if let Some(err) = first_error {
            return Err(err);
        }
    }

    events.push(mjai::Event::EndGame);
    Ok(events)
}

fn tenhou_kyoku_to_mjai_events(
    kyoku: &tenhou::Kyoku,
    backtracks: &mut HashMap<usize, BackTrack>,
) -> Result<Vec<mjai::Event>> {
    let mut events = vec![];

    // First of all, transform all takes and discards to events.
    let mut take_events = (0..4)
        .map(|i| {
            take_action_to_events(i, &kyoku.action_tables[i as usize].takes)
                .map(|ev| ev.into_iter().peekable())
        })
        .collect::<Result<Vec<_>>>()?;
    let mut discard_events = (0..4)
        .map(|i| {
            discard_action_to_events(i, &kyoku.action_tables[i as usize].discards)
                .map(|ev| ev.into_iter().peekable())
        })
        .collect::<Result<Vec<_>>>()?;

    // Then emit the events in order.
    let oya = kyoku.meta.kyoku_num % 4;
    let bakaze = match kyoku.meta.kyoku_num / 4 {
        0 => Pai::East,
        1 => Pai::South,
        2 => Pai::West,
        _ => Pai::North,
    };
    let mut dora_feed = kyoku.dora_indicators.clone().into_iter();
    let mut reach_flag: Option<usize> = None;
    let mut last_tsumo = Pai::Unknown;
    let mut last_dahai = Pai::Unknown;
    let mut last_actor: Option<u8> = None;
    let mut need_new_dora = false;

    events.push(mjai::Event::StartKyoku {
        bakaze,
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

    let mut actor = oya as usize;
    for idx in 0.. {
        // Start to process a take event.
        let take = take_events[actor]
            .next()
            .ok_or(ConvertError::InsufficientTakes {
                kyoku: kyoku.meta.kyoku_num,
                honba: kyoku.meta.honba,
                actor: actor as u8,
            })?;

        // Record the pai so that it can be filled in tsumogiri dahai.
        if let mjai::Event::Tsumo { pai, .. } = take {
            last_tsumo = pai;
        } else if let Some((target, pai)) = take.naki_info() {
            if pai != last_dahai
                || last_actor
                    .filter(|&a| a != target || a == actor as u8)
                    .is_some()
            {
                return Err(ConvertError::UnexpectedNaki {
                    action: take,
                    last_dahai,
                    last_actor,
                    kyoku: kyoku.meta.kyoku_num,
                    honba: kyoku.meta.honba,
                    actor: actor as u8,
                });
            }
        }

        // If a reach event was emitted before, set it as accepted now.
        if let Some(actor) = reach_flag.take() {
            events.push(mjai::Event::ReachAccepted { actor: actor as u8 });
        }

        // If the take is daiminkan, skip one discard and immediately consume
        // the next take event from the same actor.
        if let mjai::Event::Daiminkan { .. } = take {
            events.push(take);
            discard_events[actor].next();
            need_new_dora = true;
            continue;
        }

        // Emit the take event.
        events.push(take);

        // Check if the kyoku ends here, can be ryukyoku (九種九牌) or tsumo.
        // Here it simply checks if there is no more discard for current actor.
        if discard_events[actor].peek().is_none() {
            end_kyoku(&mut events, kyoku);
            break;
        }

        // Start to process a discard event.
        let discard = discard_events[actor]
            .next()
            .ok_or(ConvertError::InsufficientDiscards {
                kyoku: kyoku.meta.kyoku_num,
                honba: kyoku.meta.honba,
                actor: actor as u8,
            })?
            .fill_possible_tsumogiri(last_tsumo);

        // Record the pai to check if someone naki it.
        if let mjai::Event::Dahai { pai, .. } = discard {
            last_dahai = pai;
        }

        // Emit the discard event.
        events.push(discard.clone());

        // Process previous minkan.
        if matches!(discard, mjai::Event::Dahai { .. }) && need_new_dora {
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

        // Process reach declare.
        //
        // A reach declare consists of two events (reach
        // + dahai).
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
            if let mjai::Event::Dahai { pai, .. } = dahai {
                last_dahai = pai;
            }
            events.push(dahai);
        }

        // Check if the kyoku ends here, can be ryukyoku or ron.
        //
        // Here it simply checks if there is no more take for every single
        // actor.
        if (0..4).all(|i| take_events[i].peek().is_none()) {
            end_kyoku(&mut events, kyoku);
            break;
        }

        // Check if the last discard was ankan or kakan.
        //
        // For kan, it will immediately consume the next take event from the
        // same actor.
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

        // Decide who is the next actor.
        //
        // For most of the time, if someone takes taki of the previous discard,
        // then it will be him, otherwise it will be the shimocha.
        //
        // There are some edge cases when there are multiple candidates for the
        // next actor, which will be handled by the second pass of the filter.
        last_actor = Some(actor as u8);
        actor = (0..4)
            .filter(|&i| i != actor)
            // First pass, filter the naki that takes the specific tile from the
            // specific target.
            .filter_map(|i| {
                if let Some(take) = take_events[i].peek() {
                    if let Some((target, pai)) = take.naki_info() {
                        if target == (actor as u8) && pai == last_dahai {
                            return Some((i, take.naki_to_ord()));
                        }
                    }
                }

                None
            })
            // Second pass, compare the nakis and filter out the final
            // candidate.
            //
            // If a Chi and a Pon that calls the same tile from the same actor
            // can take place at the same time, then Pon must be the first to
            // take place, because if the Chi is the first instead, then the Pon
            // will be impossible to take as he will have no chance to Pon from
            // the same actor without Tsumo first.
            //
            // There is one exception to make the Chi legal though - the actor
            // takes another naki (Pon) before him, which is rare to be seen and
            // it seems not possible to properly describe it on tenhou.net/6.
            .max_by_key(|&(_, naki_ord)| naki_ord)
            .map(|(i, _)| i)
            // Backtracking, mitigate the real-naki-of-two-identical-discard
            // problem. If you are wondering, check `confusing_nakis` in
            // testdata and load them into tenhou.net/6 to see what the problem
            // is.
            //
            // Basically, the condition of such problem to occur is when actor A
            // discard the exact same pai at the next step, without giving actor
            // B any chance to tsumo, while actor B actually pon'd this pai. In
            // the end, we are not sure which one of the two identical dahais
            // actor A make is corresponding to actor B's pon.
            //
            // I really can't think of a better way to solve this.
            .and_then(|i| {
                match discard_events[actor].peek() {
                    // There is no more discard for this actor, so no chance for
                    // the problem to exist.
                    None => Some(i),

                    Some(next) => {
                        let mut process_dahai = |pai| {
                            match pai {
                                // tsumogiri
                                Pai::Unknown => {
                                    match take_events[actor].peek() {
                                        // Fill the tsumogiri dahai with tsumo pai
                                        Some(&mjai::Event::Tsumo { pai, .. }) => Some(pai),
                                        // tsumogiri must have a tsumo, not pon or chi
                                        _ => unreachable!(),
                                    }
                                }

                                // tedashi, can be after chi or pon
                                _ => Some(pai),
                            }
                        };

                        let dahai = match *next {
                            mjai::Event::Dahai { pai, .. } => process_dahai(pai),
                            mjai::Event::Reach { .. } => {
                                let mut cloned = discard_events[actor].clone();
                                if let Some(mjai::Event::Dahai { pai, .. }) = cloned.nth(1) {
                                    process_dahai(pai)
                                } else {
                                    // dahai is the only possible event after reach
                                    unreachable!()
                                }
                            }

                            // The next discard event is ankan or kakan
                            _ => None,
                        };

                        match dahai {
                            // Condition: if the actor discard the exact same
                            // pai at the next step, which leads to all the
                            // confusions.
                            Some(pai) if pai == last_dahai => match backtracks.entry(idx) {
                                Entry::Vacant(v) => {
                                    // Try taking the first dahai as the real
                                    // naki.
                                    v.insert(BackTrack {
                                        use_the_first_branch: true,
                                    });
                                    Some(i)
                                }

                                Entry::Occupied(mut o) => {
                                    // This is where the backtrack happens.
                                    let mut bc = o.get_mut();
                                    if bc.use_the_first_branch {
                                        // When this branch is reached, it is
                                        // likely the first branch has failed,
                                        // that is, the real naki doesn't seem
                                        // to be the first discard, so we will
                                        // try the second discard.
                                        bc.use_the_first_branch = false;
                                    } else {
                                        // Both branches are wrong, backtrack
                                        // further to the previous point of
                                        // divergence.
                                        //
                                        // None is still returned here to
                                        // trigger an error at the end of the
                                        // outer function so that the backtrack
                                        // can continue.
                                        o.remove_entry();
                                    }
                                    None
                                }
                            },

                            // The next discard event is ankan or kakan
                            _ => Some(i),
                        }
                    }
                }
            })
            .unwrap_or((actor + 1) % 4);
    }

    Ok(events)
}

fn take_action_to_events(actor: u8, takes: &[tenhou::ActionItem]) -> Result<Vec<mjai::Event>> {
    takes
        .iter()
        .map(|take| match take {
            tenhou::ActionItem::Tsumogiri(_) => Err(ConvertError::UnexpectedTsumogiri),

            &tenhou::ActionItem::Pai(pai) => Ok(mjai::Event::Tsumo { actor, pai }),

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
                        consumed: mjai::Consumed2::from([
                            pai_from_bytes(&naki[3..5])?,
                            pai_from_bytes(&naki[5..7])?,
                        ]),
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
                            consumed: mjai::Consumed2::from([
                                pai_from_bytes(&naki[3..5])?,
                                pai_from_bytes(&naki[5..7])?,
                            ]),
                        }),

                        // from toimen
                        // e.g. "12p1212" => pon 2m from toimen
                        2 => Ok(mjai::Event::Pon {
                            actor,
                            target: (actor + 2) % 4,
                            pai: pai_from_bytes(&naki[3..5])?,
                            consumed: mjai::Consumed2::from([
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[5..7])?,
                            ]),
                        }),
                        // from shimocha
                        // e.g. "3737p37" => pon 7s from shimocha
                        4 => Ok(mjai::Event::Pon {
                            actor,
                            target: (actor + 1) % 4,
                            pai: pai_from_bytes(&naki[5..7])?,
                            consumed: mjai::Consumed2::from([
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[2..4])?,
                            ]),
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
                            consumed: mjai::Consumed3::from([
                                pai_from_bytes(&naki[3..5])?,
                                pai_from_bytes(&naki[5..7])?,
                                pai_from_bytes(&naki[7..9])?,
                            ]),
                        }),

                        // from toimen
                        // e.g. "26m262626" => kan 6p from toimen
                        2 => Ok(mjai::Event::Daiminkan {
                            actor,
                            target: (actor + 2) % 4,
                            pai: pai_from_bytes(&naki[3..5])?,
                            consumed: mjai::Consumed3::from([
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[5..7])?,
                                pai_from_bytes(&naki[7..9])?,
                            ]),
                        }),

                        // from shimocha
                        // e.g. "131313m13" => kan 3m from shimocha
                        6 => Ok(mjai::Event::Daiminkan {
                            actor,
                            target: (actor + 1) % 4,
                            pai: pai_from_bytes(&naki[7..9])?,
                            consumed: mjai::Consumed3::from([
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[2..4])?,
                                pai_from_bytes(&naki[4..6])?,
                            ]),
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
            &tenhou::ActionItem::Pai(pai) => {
                let ev = mjai::Event::Dahai {
                    actor,
                    pai,
                    tsumogiri: false,
                };

                ret.push(ev);
            }

            tenhou::ActionItem::Tsumogiri(_) => {
                let ev = mjai::Event::Dahai {
                    actor,
                    pai: Pai::Unknown, // must be filled later
                    tsumogiri: true,
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
                            consumed: mjai::Consumed3::from([
                                pai_from_bytes(&naki[3..5])?,
                                pai_from_bytes(&naki[5..7])?,
                                pai_from_bytes(&naki[7..9])?,
                            ]),
                        },

                        // previously pon from toimen
                        // e.g. "41k414141" => pon 1z from toimen then kan
                        2 => mjai::Event::Kakan {
                            actor,
                            pai: pai_from_bytes(&naki[3..5])?,
                            consumed: mjai::Consumed3::from([
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[5..7])?,
                                pai_from_bytes(&naki[7..9])?,
                            ]),
                        },

                        // previously pon from shimocha
                        // e.g. "4646k4646" => pon 6z from shimocha then kan
                        4 => mjai::Event::Kakan {
                            actor,
                            pai: pai_from_bytes(&naki[5..7])?,
                            consumed: mjai::Consumed3::from([
                                pai_from_bytes(&naki[0..2])?,
                                pai_from_bytes(&naki[2..4])?,
                                pai_from_bytes(&naki[7..9])?,
                            ]),
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
                        consumed: mjai::Consumed4::from([
                            pai_from_bytes(&naki[0..2])?,
                            pai_from_bytes(&naki[2..4])?,
                            pai_from_bytes(&naki[4..6])?,
                            pai,
                        ]),
                    };

                    ret.push(ev);
                } else {
                    // reach
                    // e.g. "r35" => discard 5s to reach

                    if naki_string.len() != 3 {
                        return Err(ConvertError::InvalidNaki(naki_string.clone()));
                    }

                    let pai = if &naki[1..3] == b"60" {
                        Pai::Unknown
                    } else {
                        pai_from_bytes(&naki[1..3])?
                    };

                    ret.push(mjai::Event::Reach { actor });
                    ret.push(mjai::Event::Dahai {
                        actor,
                        pai, // must be filled later if it is tsumogiri
                        tsumogiri: pai == Pai::Unknown,
                    });
                }
            }
        };
    }

    Ok(ret)
}

fn end_kyoku(events: &mut Vec<mjai::Event>, kyoku: &tenhou::Kyoku) {
    match &kyoku.end_status {
        tenhou::kyoku::EndStatus::Hora { details } => {
            events.extend(details.iter().map(|detail| mjai::Event::Hora {
                actor: detail.who,
                target: detail.target,
                deltas: Some(detail.score_deltas),
            }));
        }

        tenhou::kyoku::EndStatus::Ryukyoku { score_deltas } => {
            events.push(mjai::Event::Ryukyoku {
                deltas: Some(*score_deltas),
            });
        }
    };

    events.push(mjai::Event::EndKyoku);
}

#[inline]
fn pai_from_bytes(b: &[u8]) -> Result<Pai> {
    let s = String::from_utf8_lossy(b);
    let id: u8 = s
        .parse()
        .map_err(|_| ConvertError::InvalidPai(s.clone().into_owned()))?;

    Pai::try_from(id).map_err(|_| ConvertError::InvalidPai(s.clone().into_owned()))
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
