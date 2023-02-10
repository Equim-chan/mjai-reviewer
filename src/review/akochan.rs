use crate::log;
use crate::state::State;
use convlog::{tile_set_eq, tu8, Event, Tile};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json as json;

#[derive(Debug, Serialize)]
pub struct Review {
    pub total_reviewed: usize,
    pub rating: f64,
    pub kyokus: Vec<KyokuReview>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct KyokuReview {
    /// In tenhou.net/6 format, counts from 0
    pub kyoku: u8,
    pub honba: u8,
    /// Must be either (multiple) Hora(s) or one Ryukyoku
    pub end_status: Vec<Event>,

    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    junme: u8,
    tiles_left: u8,
    last_actor: u8,
    tile: Tile,

    state: State,
    /// Always false for akochan
    at_self_chi_pon: bool,
    /// Always false for akochan
    at_self_riichi: bool,
    at_opponent_kakan: bool,

    /// At most 2 events (tuple)
    expected: Vec<Event>,
    /// At most 2 events (tuple)
    actual: Vec<Event>,
    details: Vec<DetailedAction>,

    acceptance: Acceptance,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum Acceptance {
    Disagree,
    Tolerable,
    Agree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DetailedAction {
    moves: Vec<Event>,
    review: Detail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Detail {
    // these Options are None iff `rule_base_flag && !ori_flag` is true in akochan
    total_houjuu_hai_prob_now: Option<f64>,
    total_houjuu_hai_value_now: Option<f64>,
    pt_exp_after: Option<f64>,
    pt_exp_total: Option<f64>,
}

pub struct Reviewer<'a> {
    pub akochan_exe: &'a Path,
    pub akochan_dir: &'a Path,
    pub tactics_config: &'a Path,
    pub events: &'a [Event],
    pub player_id: u8,
    pub deviation_threshold: f64,
    pub verbose: bool,
}

impl Reviewer<'_> {
    pub fn review(&self) -> Result<Review> {
        let &Self {
            akochan_exe,
            akochan_dir,
            tactics_config,
            events,
            player_id,
            deviation_threshold,
            verbose,
        } = self;

        let mut kyoku_reviews = vec![];

        let player_id_string = player_id.to_string();
        let args = &[
            "pipe_detailed".as_ref(),
            tactics_config,
            player_id_string.as_ref(),
        ];

        if verbose {
            log!("$ cd {akochan_dir:?}");
            log!(
                "$ {akochan_exe:?}{}",
                args.iter()
                    .fold("".to_owned(), |acc, p| format!("{acc} {p:?}")),
            );
        }

        let mut akochan = Command::new(akochan_exe)
            .args(args)
            .current_dir(Path::new(akochan_dir))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to spawn engine")?;

        let mut stdin = akochan
            .stdin
            .take()
            .context("failed to get stdin of engine")?;
        let stdout = akochan
            .stdout
            .take()
            .context("failed to get stdout of engine")?;
        let mut stdout_lines = BufReader::new(stdout).lines();

        let events_len = events.len();
        let mut total_reviewed = 0;
        let mut total_tolerated = 0;
        let mut total_problems = 0;
        let mut raw_rating = 0.;

        let mut kyoku_review = KyokuReview::default();
        let mut state = State::new(player_id);
        let mut junme = 0;
        let mut tiles_left = 70;
        let mut entries = vec![];
        let mut is_riichied = false;

        for (i, event) in events.iter().enumerate() {
            let to_write = json::to_string(event).unwrap();
            writeln!(stdin, "{to_write}").context("failed to write to engine")?;
            if verbose {
                log!("> {to_write}");
            }

            // update the state
            state.update(event).context("failed to update state")?;

            // this match does two things:
            // 1. setting board metadata like bakaze, kyoku, honba, junme
            // 2. decide whether or not this event is a valid timing when we can review
            match *event {
                Event::StartKyoku {
                    bakaze,
                    kyoku: kk,
                    honba,
                    ..
                } => {
                    let kyoku = (bakaze.as_u8() - tu8!(E)) * 4 + kk - 1;
                    kyoku_review.kyoku = kyoku;
                    kyoku_review.honba = honba;
                    is_riichied = false;
                    tiles_left = 70;

                    continue;
                }

                Event::EndKyoku => {
                    kyoku_review.entries = entries.clone();
                    entries.clear();

                    kyoku_reviews.push(kyoku_review.clone());
                    kyoku_review = KyokuReview::default();

                    junme = 0;
                    continue;
                }

                Event::Hora { .. } | Event::Ryukyoku { .. } => {
                    kyoku_review.end_status.push(event.clone());
                    continue;
                }

                Event::Dahai { actor, .. } | Event::Kakan { actor, .. } => {
                    if actor == player_id {
                        continue;
                    }
                }

                Event::Tsumo { actor, .. } => {
                    tiles_left -= 1;
                    if actor != player_id {
                        continue;
                    }
                    junme += 1;
                }

                Event::Chi { actor, .. } | Event::Pon { actor, .. } => {
                    if actor == player_id {
                        junme += 1;
                    }
                    continue;
                }

                Event::ReachAccepted { actor } => {
                    if actor == player_id {
                        is_riichied = true;
                    }
                    continue;
                }

                _ => continue,
            };

            log!(
                "reviewing kyoku {} honba {} junme {} ({:.2}%)",
                kyoku_review.kyoku,
                kyoku_review.honba,
                junme,
                (i as f32) / (events_len as f32) * 100.,
            );

            // should have at least 4, e.g. dahai -> ryukyoku -> end_kyoku -> end_game
            if events.len() < i + 4 {
                bail!("wrong size of input events, expected to have 4 more");
            }

            // be careful, stdout_lines.next() may block.
            let line = stdout_lines
                .next()
                .context("failed to read from engine: unexpected EOF")?
                .context("failed to read from engine")?;
            if verbose {
                log!("< {}", line.trim());
            }

            let actions: Vec<DetailedAction> =
                json::from_str(&line).context("failed to parse JSON output of engine")?;

            if actions.is_empty() || actions.iter().any(|a| a.moves.is_empty()) {
                log!("WARNING: actions or some moves in actions is empty");
                continue;
            }

            // skip the comparison when
            // 1. it is not our turn and there is no chance to naki
            // 2. our state is reached and only tsumogiri is possible
            let actual_action = next_action_for_compare(&events[(i + 1)..]);
            if actions.len() == 1 && (is_riichied || actions[0].moves[0] == Event::None) {
                match actual_action[0] {
                    // akochan don't give any recommended action,
                    // but naki happen
                    Event::Pon { actor, .. } | Event::Chi { actor, .. } => {
                        if actor != player_id {
                            continue;
                        }
                        // continue to review if the player naki
                    }
                    _ => continue,
                }
            }

            let expected_action = &actions[0].moves; // best move
            let is_equal_or_innocent = compare_action(actual_action, expected_action, player_id)
                .context("invalid state in event")?;
            let actual_action_strict = next_action_strict(actual_action, player_id);

            let (move_rating, acceptance) = if is_equal_or_innocent {
                (1., Acceptance::Agree) // it is an acceptable move
            } else if deviation_threshold <= 0. {
                (1., Acceptance::Disagree) // not acceptable and no threshold set, deny
            } else if let Some(expected_ev) = actions[0].review.pt_exp_total {
                // this is O(n)
                // ;(
                let lookup = actions
                    .iter()
                    .find(|&ex| compare_action_strict(&actual_action_strict, &ex.moves))
                    .map(|detail| detail.review.pt_exp_total);

                let min_ev = actions
                    .last()
                    .unwrap() // actions[0] is already asserted
                    .review
                    .pt_exp_total
                    .context("invalid message, pt_exp_total is None when it shouldn't")?;

                match lookup {
                    None => {
                        // Usually it is some kind of kan. This is a known issue of akochan.
                        // It can be mitigated by setting `do_kan_ordinary` to true in tactics.json
                        log!(
                            "WARNING: unable to find player's action in akochan's return,
                            expected to find: {actual_action_strict:?}, list: {:?}",
                            actions.iter().map(|a| a.moves.clone()).collect::<Vec<_>>(),
                        );
                        // Skip this situation as it is unclear for akochan, probably not what
                        // those who set --deviation-threshold expect.
                        continue;
                    }

                    Some(Some(actual_ev)) => {
                        let range = expected_ev - min_ev;
                        let error = expected_ev - actual_ev;
                        let rating = if range > 0. { 1. - error / range } else { 1. };

                        let dev = expected_ev - actual_ev;
                        if dev <= deviation_threshold {
                            if verbose {
                                log!(
                                    "expected_ev - actual_ev <= deviation_threshold
                                    ({expected_ev} - {actual_ev} = {dev} < {deviation_threshold})",
                                );
                            }
                            (rating, Acceptance::Tolerable) // not acceptable but tolerable
                        } else {
                            (rating, Acceptance::Disagree) // not acceptable, the threshold is set but the value is lower than it
                        }
                    }

                    Some(None) => {
                        // Early turn or high shanten, see `rule_base_flag && !ori_flag` in
                        // akochan:ai_src/selector.cpp.
                        // Skip this situation as it is very likely a small difference,
                        // probably not what those who set --deviation-threshold expect.
                        (1., Acceptance::Agree)
                    }
                }
            } else {
                // Ditto for early turn or high shanten, akochan don't
                // give any recommended action.
                match actual_action[0] {
                    // naki happen under this situation usually is not recommended
                    Event::Pon { .. } | Event::Chi { .. } => (1., Acceptance::Disagree),
                    // Agree for other actions cause it is very likely a small difference
                    _ => (1., Acceptance::Agree),
                }
            };

            // handle kakan
            let (last_actor, tile, at_opponent_kakan) = match *event {
                Event::Dahai { actor, pai, .. } | Event::Tsumo { actor, pai, .. } => {
                    (actor, pai, false)
                }
                Event::Kakan { actor, pai, .. } => (actor, pai, true),
                _ => {
                    bail!("invalid state: no actor or tile found, event: {event:?}");
                }
            };

            match acceptance {
                Acceptance::Disagree => total_problems += 1,
                Acceptance::Tolerable => total_tolerated += 1,
                Acceptance::Agree => (),
            };
            total_reviewed += 1;
            raw_rating += move_rating;

            let entry = Entry {
                junme,
                tiles_left,
                last_actor,
                tile,
                state: state.clone(),
                at_self_chi_pon: false,
                at_self_riichi: false,
                at_opponent_kakan,
                expected: expected_action.clone(),
                actual: actual_action_strict,
                details: actions,
                acceptance,
            };
            log!(
                "review entry created: {acceptance:?}
                ({total_problems}/{total_tolerated}/{total_reviewed}, {:.03})",
                (raw_rating / total_reviewed as f64).powi(2) * 100.,
            );
            if verbose {
                log!("{:?}", entry);
            }

            entries.push(entry);
        }
        drop(stdin);

        let status = akochan.wait()?;
        if !status.success() {
            if let Some(code) = status.code() {
                bail!("non-zero exit code: {code}");
            }
            bail!("process terminated by signal");
        }

        Ok(Review {
            total_reviewed,
            rating: (raw_rating / total_reviewed as f64).powi(2),
            kyokus: kyoku_reviews,
        })
    }
}

fn next_action_for_compare(events: &[Event]) -> &[Event] {
    match events[0] {
        Event::Dora { .. } | Event::ReachAccepted { .. } => next_action_for_compare(&events[1..]),
        Event::Hora { .. } => &events[..3], // considering multiple rons
        Event::Chi { .. } | Event::Pon { .. } | Event::Reach { .. } => &events[..2],
        _ => &events[..1],
    }
}

/// Get actual action from player's perspective, which will handle
/// Event::None and multiple Event::Hora properly.
///
/// `rough_action` must be the return value of `next_action_for_compare`.
fn next_action_strict(rough_action: &[Event], player_id: u8) -> Vec<Event> {
    match rough_action[0] {
        // passed when it's supposed to naki
        Event::Tsumo { .. } => vec![Event::None],

        // filter the actor's hora from multiple horas
        Event::Hora { .. } => vec![rough_action
            .iter()
            .take(3)
            .find(|&a| matches!(*a, Event::Hora { actor, .. } if actor == player_id))
            .cloned()
            .unwrap_or(Event::None)],

        _ => match rough_action[0].actor() {
            // not the target actor, who did nothing (passed a possible naki)
            Some(actual_actor) if actual_actor != player_id => vec![Event::None],

            // anything else the target actor did
            _ => rough_action.to_vec(),
        },
    }
}

/// Returns true if actual_action is the same as expected_action.
fn compare_action_strict(actual_action: &[Event], expected_action: &[Event]) -> bool {
    expected_action
        .iter()
        .zip(actual_action)
        .all(|(e, a)| match (e, a) {
            // ignore `tsumogiri`
            (Event::Dahai { pai: ee, .. }, Event::Dahai { pai: aa, .. }) => ee == aa,
            // ignore `delta`
            (Event::Hora { actor: ee, .. }, Event::Hora { actor: aa, .. }) => ee == aa,
            _ => e == a,
        })
}

/// Returns true if actual_action is innocent or the same as expected_action.
fn compare_action(
    actual_action: &[Event],
    expected_action: &[Event],
    player_id: u8,
) -> Result<bool> {
    // hot path.
    //
    // note that for Event::Dahai, it also compares tsumogiri, therefore if they
    // choose same pai's in Dahai but tsumogiri's are different, it will fall
    // into the slow path.
    if expected_action
        .iter()
        .zip(actual_action)
        .all(|(e, a)| e == a)
    {
        return Ok(true);
    }

    // fallback to slow path.
    let actual = &actual_action[0];

    match expected_action[0] {
        Event::Dahai { pai, .. } => {
            match *actual {
                // ignore the difference of tsumogiri
                Event::Dahai {
                    pai: actual_pai, ..
                } => Ok(actual_pai == pai),
                _ => Ok(false),
            }
        }

        Event::Ankan { consumed, .. } => match *actual {
            Event::Ankan {
                consumed: actual_consumed,
                ..
            } => Ok(tile_set_eq(&actual_consumed, &consumed, false)),
            _ => Ok(false),
        },

        Event::Kakan { pai, .. } => match *actual {
            Event::Kakan {
                pai: actual_pai, ..
            } => Ok(actual_pai == pai),
            _ => Ok(false),
        },

        Event::Reach { .. } => {
            match actual {
                Event::Reach { .. } => {
                    let next_actual = actual_action.get(1);

                    // ignore the difference of tsumogiri
                    if let Some(&Event::Dahai {
                        pai: actual_pai, ..
                    }) = next_actual
                    {
                        let next_expected = expected_action.get(1);

                        if let Some(&Event::Dahai { pai, .. }) = next_expected {
                            Ok(actual_pai == pai)
                        } else {
                            bail!("event after Reach is not Dahai, got {next_expected:?}");
                        }
                    } else {
                        bail!("event after Reach is not Dahai, got {next_actual:?}");
                    }
                }
                _ => Ok(false),
            }
        }

        Event::Chi { consumed, .. } => {
            match *actual {
                Event::Tsumo { .. } => Ok(false),
                Event::Chi {
                    consumed: actual_consumed,
                    ..
                } if tile_set_eq(&actual_consumed, &consumed, false) => {
                    let next_actual = actual_action.get(1);

                    if let Some(&Event::Dahai {
                        pai: actual_pai, ..
                    }) = next_actual
                    {
                        let next_expected = expected_action.get(1);

                        if let Some(&Event::Dahai { pai, .. }) = next_expected {
                            Ok(actual_pai == pai)
                        } else {
                            bail!("event after Chi is not Dahai, got {next_expected:?}");
                        }
                    } else {
                        bail!("event after Chi is not Dahai, got {next_actual:?}");
                    }
                }
                _ => {
                    if let Some(actor) = actual.actor() {
                        // interrupted by opponent's pon, daiminkan or ron
                        Ok(actor != player_id)
                    } else {
                        bail!("unexpected event: {actual:?}");
                    }
                }
            }
        }

        Event::Pon { consumed, .. } => {
            match *actual {
                Event::Tsumo { .. } => Ok(false),
                Event::Pon {
                    consumed: actual_consumed,
                    ..
                } if tile_set_eq(&actual_consumed, &consumed, false) => {
                    let next_actual = actual_action.get(1);

                    if let Some(&Event::Dahai {
                        pai: actual_pai, ..
                    }) = next_actual
                    {
                        let next_expected = expected_action.get(1);

                        if let Some(&Event::Dahai { pai, .. }) = next_expected {
                            Ok(actual_pai == pai)
                        } else {
                            bail!("event after Pon is not Dahai, got {next_expected:?}");
                        }
                    } else {
                        bail!("event after Pon is not Dahai, got {next_actual:?}");
                    }
                }
                _ => {
                    if let Some(actor) = actual.actor() {
                        // interrupted by opponent's ron
                        Ok(actor != player_id)
                    } else {
                        bail!("unexpected event: {actual:?}");
                    }
                }
            }
        }

        Event::Daiminkan { .. } => {
            match actual {
                Event::Tsumo { .. } => Ok(false),
                Event::Daiminkan { .. } => Ok(true),
                _ => {
                    if let Some(actor) = actual.actor() {
                        // interrupted by opponent's ron
                        Ok(actor != player_id)
                    } else {
                        bail!("unexpected event: {actual:?}");
                    }
                }
            }
        }

        // considering multiple rons
        Event::Hora { .. } => Ok(actual_action.iter().take(3).any(|a| {
            if let Event::Hora { actor, .. } = *a {
                actor == player_id
            } else {
                false
            }
        })),

        Event::None => match actual {
            // issue #19
            Event::Tsumo { .. } | Event::Ryukyoku { .. } => Ok(true),
            _ => {
                if let Some(actor) = actual.actor() {
                    Ok(actor != player_id)
                } else {
                    bail!("unexpected event: {actual:?}");
                }
            }
        },

        Event::Ryukyoku { .. } => match actual {
            Event::Ryukyoku { .. } => Ok(true),
            _ => Ok(false),
        },

        _ => bail!("unexpected event: {actual:?}"),
    }
}
