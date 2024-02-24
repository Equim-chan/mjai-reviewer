use crate::log;
use crate::softmax::softmax;
use crate::state::State;
use convlog::{must_tile, t, tile_set_eq, tu8, Event, Tile};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{array, mem};

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json as json;
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize)]
pub struct Review {
    pub total_reviewed: usize,
    pub total_matches: usize,
    pub rating: f64,
    pub temperature: f32,
    pub kyokus: Vec<KyokuReview>,

    pub relative_phi_matrix: Vec<[[f64; 4]; 4]>,
    pub model_tag: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct KyokuReview {
    /// In tenhou.net/6 format, counts from 0
    pub kyoku: u8,
    pub honba: u8,
    /// Must be either (multiple) Hora(s) or one Ryukyoku
    pub end_status: Vec<Event>,
    pub relative_scores: [i32; 4],

    pub entries: Vec<Entry>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    junme: u8,
    tiles_left: u8,
    last_actor: u8,
    tile: Tile,

    state: State,
    at_self_chi_pon: bool,
    at_self_riichi: bool,
    at_opponent_kakan: bool,

    expected: Event,
    actual: Event,
    is_equal: bool,
    details: Vec<Detail>,

    shanten: i8,
    at_furiten: bool,
    actual_index: usize,
}

#[derive(Debug, Clone, Serialize)]
struct Detail {
    action: Event,
    q_value: f32,
    prob: f32,

    // not displayed, for sorting only
    #[serde(skip)]
    label: Label,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum Label {
    General(usize),
    KanSelect(usize),
}

#[derive(Debug, Clone, Deserialize)]
struct RawAction {
    #[serde(flatten)]
    event: Event,
    meta: Option<Metadata>,
}

// From /libriichi/src/mjai.rs
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Metadata {
    q_values: Option<Vec<f32>>,
    mask_bits: Option<u64>,
    is_greedy: Option<bool>,
    batch_size: Option<usize>,
    eval_time_ns: Option<u64>,
    shanten: Option<i8>,
    at_furiten: Option<bool>,
    kan_select: Option<Box<Metadata>>,
}

#[derive(Deserialize)]
struct ExtraData {
    model_tag: String,
    phi_matrix: Vec<[[f64; 4]; 4]>,
}

pub struct Reviewer<'a> {
    pub mortal_exe: &'a Path,
    pub mortal_cfg: &'a Path,
    pub events: &'a [Event],
    pub player_id: u8,
    pub temperature: f32,
    pub verbose: bool,
}

impl Reviewer<'_> {
    pub fn review(&self) -> Result<Review> {
        let &Self {
            mortal_exe,
            mortal_cfg,
            events,
            player_id,
            temperature,
            verbose,
        } = self;

        let mut kyoku_reviews = vec![];

        if verbose {
            log!("$ env MORTAL_REVIEW_MODE=1 MORTAL_CFG={mortal_cfg:?} {mortal_exe:?} {player_id}");
        }

        let mut mortal = Command::new(mortal_exe)
            .arg(player_id.to_string())
            .env("MORTAL_REVIEW_MODE", "1")
            .env("MORTAL_CFG", mortal_cfg)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to spawn engine")?;

        let mut stdin = mortal
            .stdin
            .take()
            .context("failed to get stdin of engine")?;
        let stdout = mortal
            .stdout
            .take()
            .context("failed to get stdout of engine")?;
        let mut stdout_lines = BufReader::new(stdout).lines();

        let events_len = events.len();
        let mut total_reviewed = 0;
        let mut total_matches = 0;
        let mut raw_rating = 0.;

        let mut kyoku_review = KyokuReview::default();
        let mut state = State::new(player_id);
        let mut junme = 0;
        let mut tiles_left = 70;
        let mut last_tsumo_or_discard = None;
        let mut last_actor = 0;
        let mut entries = vec![];

        for (i, event) in events.iter().enumerate() {
            let to_write = json::to_string(event).unwrap();
            writeln!(stdin, "{to_write}").context("failed to write to engine")?;
            if verbose {
                log!("> {to_write}");
            }

            // update the state
            state.update(event).context("failed to update state")?;

            let mut at_self_chi_pon = false;
            let mut at_self_riichi = false;
            match *event {
                Event::StartKyoku {
                    bakaze,
                    kyoku: kk,
                    honba,
                    scores,
                    ..
                } => {
                    kyoku_review.kyoku = (bakaze.as_u8() - tu8!(E)) * 4 + kk - 1;
                    kyoku_review.honba = honba;
                    kyoku_review.relative_scores = scores;
                    kyoku_review.relative_scores.rotate_left(player_id as usize);
                    tiles_left = 70;
                }

                Event::EndKyoku => {
                    kyoku_review.entries = mem::take(&mut entries);
                    kyoku_reviews.push(mem::take(&mut kyoku_review));
                    junme = 0;
                }

                Event::Hora { .. } | Event::Ryukyoku { .. } => {
                    kyoku_review.end_status.push(event.clone());
                }

                Event::Tsumo { actor, pai, .. } => {
                    if actor == player_id {
                        last_tsumo_or_discard = Some(pai);
                        junme += 1;
                    }
                    tiles_left -= 1;
                }

                Event::Chi { actor, .. } | Event::Pon { actor, .. } if actor == player_id => {
                    at_self_chi_pon = true;
                    junme += 1;
                }

                Event::Reach { actor } if actor == player_id => {
                    at_self_riichi = true;
                }

                Event::Dahai { pai, .. } | Event::Kakan { pai, .. } => {
                    last_tsumo_or_discard = Some(pai);
                }

                _ => (),
            }

            if let Some(actor) = event.actor() {
                last_actor = actor;
            }

            let line = stdout_lines
                .next()
                .context("failed to read from engine: unexpected EOF")?
                .context("failed to read from engine")?;
            if verbose {
                log!("< {line}");
            }

            if matches!(
                event,
                Event::StartGame { .. }
                    | Event::StartKyoku { .. }
                    | Event::EndKyoku
                    | Event::EndGame
            ) {
                continue;
            }
            log!(
                "reviewing kyoku {}, honba {}, junme {}, ({:.2}%)",
                kyoku_review.kyoku,
                kyoku_review.honba,
                junme,
                i as f32 / events_len as f32 * 100.,
            );

            let output: RawAction =
                json::from_str(&line).context("failed to parse JSON output of engine")?;

            let Some(meta) = output.meta else { continue };
            let mask_bits = meta.mask_bits.context("missing mask_bits")?;
            if mask_bits.count_ones() <= 1 {
                // cannot act, or there is only one candidate
                continue;
            }
            let masks = masks_from_bits(mask_bits);
            let can_pon_or_daiminkan = masks[41] || masks[42];
            let can_agari = masks[43];
            let can_ryukyoku = masks[44];

            let Some(actual) = next_action(
                &events[i + 1..],
                player_id,
                can_pon_or_daiminkan,
                can_agari,
                can_ryukyoku,
            ) else {
                // interrupted
                continue;
            };
            let actual_label = to_label(&actual);
            ensure!(masks[actual_label], "{actual:?} is not a valid reaction");
            let mut actual_q_value_opt = None;

            let shanten = meta.shanten.context("missing shanten")?;
            let at_furiten = meta.at_furiten.context("missing at_furiten")?;
            let mut q_values = meta.q_values.context("missing q_values")?;
            let mut details = Vec::with_capacity(q_values.len());
            let mut min = f64::MAX;
            let mut max = f64::MIN;
            for (label, m) in masks.into_iter().enumerate().rev() {
                if !m {
                    continue;
                }
                let q_value = q_values.pop().context("q_values vec underflow")?;
                min = min.min(q_value as f64);
                max = max.max(q_value as f64);

                let action = to_event(&state, label, last_actor, last_tsumo_or_discard, false)?;
                if label == actual_label {
                    actual_q_value_opt = Some(q_value as f64);
                }

                details.push(Detail {
                    action,
                    q_value,
                    prob: 0., // will be calculated later
                    label: Label::General(label),
                });
            }

            let actual_kan_label = to_kan_label(&actual);
            if let Some(kan_select) = meta.kan_select {
                let mask_bits = kan_select.mask_bits.context("missing mask_bits")?;
                let num_kans = mask_bits.count_ones();
                ensure!(
                    num_kans > 0,
                    "expected `num_kans > 0`, got mask_bits = {mask_bits}",
                );

                let (orig_kan_idx, orig_kan_q_value) = details
                    .iter()
                    .enumerate()
                    .find_map(|(i, d)| {
                        matches!(d.action, Event::Ankan { .. }).then_some((i, d.q_value))
                    })
                    .context("in kan_select but no kan found in root")?;
                details.remove(orig_kan_idx);

                let masks = masks_from_bits(mask_bits);
                let mut q_values = kan_select.q_values.context("missing q_values")?;
                for (kan_label, m) in masks.into_iter().enumerate().rev() {
                    if !m {
                        continue;
                    }
                    let q_value = if num_kans == 1 {
                        orig_kan_q_value
                    } else {
                        q_values.pop().context("q_values vec underflow")?
                    };
                    min = min.min(q_value as f64);
                    max = max.max(q_value as f64);

                    let action =
                        to_event(&state, kan_label, last_actor, last_tsumo_or_discard, true)?;
                    // I REALLY want to get rid of the rating and the table...
                    if num_kans > 1 && matches!(actual_kan_label, Some(l) if l == kan_label) {
                        actual_q_value_opt = Some(q_value as f64);
                    }

                    details.push(Detail {
                        action,
                        q_value,
                        prob: 0., // will be calculated later
                        label: Label::KanSelect(kan_label),
                    });
                }
            }

            let mut probs: Vec<_> = details.iter().map(|d| d.q_value).collect();
            softmax(&mut probs, temperature);
            details.iter_mut().zip(probs).for_each(|(d, v)| d.prob = v);

            // this sort is better to be stable
            details.sort_by(|l, r| r.q_value.total_cmp(&l.q_value));
            let actual_index = details
                .iter()
                .enumerate()
                .find(|(_, d)| match (d.label, actual_kan_label) {
                    (Label::General(l), None) => l == actual_label,
                    (Label::KanSelect(l), Some(kan_label)) => l == kan_label,
                    _ => false,
                })
                .map(|(i, _)| i)
                .with_context(|| {
                    format!("failed to find action ({actual_label}, {actual_kan_label:?}) in details {details:?}")
                })?;

            let is_equal = equal_ignore_aka_consumed(&output.event, &actual);
            let actual_q_value = actual_q_value_opt
                .with_context(|| format!("failed to find q value of actual action {actual:?}"))?;
            if is_equal {
                raw_rating += 1.;
                total_matches += 1;
            } else {
                raw_rating += (actual_q_value - min) / (max - min).max(1e-6);
            }
            total_reviewed += 1;

            let tile = last_tsumo_or_discard.context("missing last tsumo or discard")?;
            let at_opponent_kakan = matches!(event, Event::Kakan { .. });

            let entry = Entry {
                junme,
                tiles_left,
                last_actor,
                tile,
                state: state.clone(),
                at_self_chi_pon,
                at_self_riichi,
                at_opponent_kakan,
                expected: output.event,
                actual,
                is_equal,
                details,
                shanten,
                at_furiten,
                actual_index,
            };
            entries.push(entry);
        }
        drop(stdin);

        let line = stdout_lines
            .next()
            .context("failed to read from engine: unexpected EOF")?
            .context("failed to read from engine")?;
        if verbose {
            log!("< {line}");
        }

        let ExtraData {
            model_tag,
            mut phi_matrix,
        } = json::from_str(&line).context("failed to parse JSON output of engine")?;
        ensure!(phi_matrix.len() == kyoku_reviews.len());

        for k in &mut phi_matrix {
            k.rotate_left(player_id as usize);
        }

        let status = mortal.wait()?;
        if !status.success() {
            if let Some(code) = status.code() {
                bail!("non-zero exit code: {code}");
            }
            bail!("process terminated by signal");
        }

        let rating = (raw_rating / total_reviewed as f64).powi(2);
        Ok(Review {
            total_reviewed,
            total_matches,
            rating,
            temperature,
            kyokus: kyoku_reviews,
            relative_phi_matrix: phi_matrix,
            model_tag,
        })
    }
}

fn masks_from_bits(bits: u64) -> [bool; 46] {
    array::from_fn(|i| (bits >> i) & 0b1 == 0b1)
}

/// It assumes they have the same actor.
fn equal_ignore_aka_consumed(a: &Event, b: &Event) -> bool {
    match (a, b) {
        (Event::Dahai { pai: l, .. }, Event::Dahai { pai: r, .. })
        | (Event::Kakan { pai: l, .. }, Event::Kakan { pai: r, .. }) => l == r,

        (Event::Chi { consumed: l, .. }, Event::Chi { consumed: r, .. })
        | (Event::Pon { consumed: l, .. }, Event::Pon { consumed: r, .. }) => {
            tile_set_eq(l, r, true)
        }

        (Event::Daiminkan { consumed: l, .. }, Event::Daiminkan { consumed: r, .. }) => {
            tile_set_eq(l, r, true)
        }
        (Event::Ankan { consumed: l, .. }, Event::Ankan { consumed: r, .. }) => {
            tile_set_eq(l, r, true)
        }

        (Event::Reach { .. }, Event::Reach { .. })
        | (Event::Hora { .. }, Event::Hora { .. })
        | (Event::Ryukyoku { .. }, Event::Ryukyoku { .. })
        | (Event::None, Event::None) => true,

        _ => false,
    }
}

fn to_label(ev: &Event) -> usize {
    match ev {
        Event::Dahai { pai, .. } => pai.as_usize(),
        Event::Reach { .. } => 37,
        Event::Chi { pai, consumed, .. } => {
            let a = consumed[0].deaka().as_usize();
            let b = consumed[1].deaka().as_usize();
            let min = a.min(b);
            let max = a.max(b);
            let x = pai.deaka().as_usize();
            if x < min {
                38
            } else if x < max {
                39
            } else {
                40
            }
        }
        Event::Pon { .. } => 41,
        Event::Daiminkan { .. } | Event::Ankan { .. } | Event::Kakan { .. } => 42,
        Event::Hora { .. } => 43,
        Event::Ryukyoku { .. } => 44,
        _ => 45,
    }
}

const fn to_kan_label(ev: &Event) -> Option<usize> {
    // Kan label is not needed for Daiminkan
    match ev {
        Event::Ankan { consumed, .. } => Some(consumed[0].deaka().as_usize()),
        Event::Kakan { pai, .. } => Some(pai.deaka().as_usize()),
        _ => None,
    }
}

/// Important note:
///
/// * For Chi and Pon, `consumed` are always deaka'd, which is different from
///   Mortal's behavior.
/// * There is no Kakan or Daiminkan, all kans will be returned as Ankan. They
///   are the same on the UI anyways.
fn to_event(
    state: &State,
    label: usize,
    target: u8,
    last_tsumo_or_discard: Option<Tile>,
    at_kan_select: bool,
) -> Result<Event> {
    let actor = state.player_id();

    if at_kan_select {
        ensure!(label < 34, "invalid kan label {label}");
        let tile = must_tile!(label);
        let consumed = [tile; 4];
        let event = Event::Ankan { actor, consumed };
        return Ok(event);
    }

    let event = match label {
        0..=36 => Event::Dahai {
            actor,
            pai: must_tile!(label),
            tsumogiri: last_tsumo_or_discard.is_some_and(|t| t.as_usize() == label),
        },
        37 => Event::Reach { actor },
        38 => {
            let pai = last_tsumo_or_discard.context("missing last discard for Chi")?;
            let can_akaize_consumed = match pai.as_u8() {
                tu8!(3m) | tu8!(4m) => state.has_tile(t!(5mr)),
                tu8!(3p) | tu8!(4p) => state.has_tile(t!(5pr)),
                tu8!(3s) | tu8!(4s) => state.has_tile(t!(5sr)),
                _ => false,
            };
            let consumed = if can_akaize_consumed {
                [pai.next().akaize(), pai.next().next().akaize()]
            } else {
                [pai.next(), pai.next().next()]
            };
            Event::Chi {
                actor,
                target,
                pai,
                consumed,
            }
        }
        39 => {
            let pai = last_tsumo_or_discard.context("missing last discard for Chi")?;
            let can_akaize_consumed = match pai.as_u8() {
                tu8!(4m) | tu8!(6m) => state.has_tile(t!(5mr)),
                tu8!(4p) | tu8!(6p) => state.has_tile(t!(5pr)),
                tu8!(4s) | tu8!(6s) => state.has_tile(t!(5sr)),
                _ => false,
            };
            let consumed = if can_akaize_consumed {
                [pai.prev().akaize(), pai.next().akaize()]
            } else {
                [pai.prev(), pai.next()]
            };
            Event::Chi {
                actor,
                target,
                pai,
                consumed,
            }
        }
        40 => {
            let pai = last_tsumo_or_discard.context("missing last discard for Chi")?;
            let can_akaize_consumed = match pai.as_u8() {
                tu8!(6m) | tu8!(7m) => state.has_tile(t!(5mr)),
                tu8!(6p) | tu8!(7p) => state.has_tile(t!(5pr)),
                tu8!(6s) | tu8!(7s) => state.has_tile(t!(5sr)),
                _ => false,
            };
            let consumed = if can_akaize_consumed {
                [pai.prev().prev().akaize(), pai.prev().akaize()]
            } else {
                [pai.prev().prev(), pai.prev()]
            };
            Event::Chi {
                actor,
                target,
                pai,
                consumed,
            }
        }
        41 => {
            let pai = last_tsumo_or_discard.context("missing last discard for Pon")?;
            let can_akaize_consumed = match pai.as_u8() {
                tu8!(5m) => state.has_tile(t!(5mr)),
                tu8!(5p) => state.has_tile(t!(5pr)),
                tu8!(5s) => state.has_tile(t!(5sr)),
                _ => false,
            };
            let consumed = if can_akaize_consumed {
                [pai.akaize(), pai.deaka()]
            } else {
                [pai.deaka(); 2]
            };
            Event::Pon {
                actor,
                target,
                pai,
                consumed,
            }
        }
        42 => {
            // in fact this is Daiminkan
            let tile = last_tsumo_or_discard.context("missing last discard for Daiminkan")?;
            let consumed = [tile, tile.deaka(), tile.deaka(), tile.deaka()];
            Event::Ankan { actor, consumed }
        }
        43 => Event::Hora {
            actor,
            target,
            deltas: None,
            ura_markers: None,
        },
        44 => Event::Ryukyoku { deltas: None },
        45 => Event::None,

        _ => bail!("unexpected label {label}"),
    };
    Ok(event)
}

/// Get actual action from player's perspective, which will handle Event::None
/// and multiple Event::Hora properly.
///
/// `None` is returned if the player actually cannot act because of some
/// interruption.
fn next_action(
    events: &[Event],
    player_id: u8,
    can_pon_or_daiminkan: bool,
    can_agari: bool,
    can_ryukyoku: bool,
) -> Option<Event> {
    let ev = &events[0];
    match ev {
        Event::Dora { .. } | Event::ReachAccepted { .. } => next_action(
            &events[1..],
            player_id,
            can_pon_or_daiminkan,
            can_agari,
            can_ryukyoku,
        ),

        // passed when it's supposed to naki
        Event::Tsumo { .. } => Some(Event::None),

        // filter the player's hora from multiple horas
        Event::Hora { .. } => events
            .iter()
            .take(3)
            .find(|&a| matches!(*a, Event::Hora { actor, .. } if actor == player_id))
            .cloned()
            .or(if can_agari {
                // actively denied to ron
                Some(Event::None)
            } else {
                None
            }),

        Event::Ryukyoku { .. } => can_ryukyoku.then(|| ev.clone()),

        _ => match ev.actor() {
            Some(actual_actor) if actual_actor != player_id => {
                // Chi, Pon, Ron etc by other players. Tsumo was already handled
                // above.
                if can_agari || can_pon_or_daiminkan {
                    // actively denied to ron, pon or daiminkan
                    Some(Event::None)
                } else {
                    None
                }
            }

            // anything else the player did
            _ => Some(ev.clone()),
        },
    }
}
