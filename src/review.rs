use crate::log;
use crate::state::State;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use convlog::mjai::Event;
use convlog::Pai;
use serde::{Deserialize, Serialize};
use serde_json as json;
use serde_with::{serde_as, DisplayFromStr};

pub struct Review {
    pub total_reviewed: usize,
    pub total_tolerated: usize,
    pub total_problems: usize,
    pub score: f64,
    pub kyokus: Vec<KyokuReview>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct KyokuReview {
    pub kyoku: u8, // in tenhou.net/6 format, counts from 0
    pub honba: u8,
    pub end_status: Vec<Event>, // must be either multiple Horas or one Ryukyoku

    pub entries: Vec<Entry>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub acceptance: Acceptance,
    pub junme: u8,
    pub actor: u8,
    #[serde_as(as = "DisplayFromStr")]
    pub pai: Pai,
    pub is_kakan: bool, // for chankan
    pub state: State,

    pub expected: Vec<Event>, // at most 2 events
    pub actual: Vec<Event>,   // at most 2 events

    pub details: Vec<DetailedAction>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Acceptance {
    Disagree,
    Tolerable,
    Agree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    // these Options are None iff `rule_base_flag && !ori_flag` is true in akochan
    total_houjuu_hai_prob_now: Option<f64>,
    total_houjuu_hai_value_now: Option<f64>,
    pt_exp_after: Option<f64>,
    pt_exp_total: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedAction {
    pub moves: Vec<Event>,
    pub review: Stat,
}

pub struct ReviewArgs<'a> {
    pub akochan_exe: &'a Path,
    pub akochan_dir: &'a Path,
    pub tactics_config: &'a Path,
    pub events: &'a [Event],
    pub target_actor: u8,
    pub deviation_threshold: f64,
    pub verbose: bool,
}

pub fn review(review_args: &ReviewArgs) -> Result<Review> {
    let &ReviewArgs {
        akochan_exe,
        akochan_dir,
        tactics_config,
        events,
        target_actor,
        deviation_threshold,
        verbose,
    } = review_args;

    let mut kyoku_reviews = vec![];

    let target_actor_string = target_actor.to_string();
    let args = &[
        "pipe_detailed".as_ref(),
        tactics_config,
        target_actor_string.as_ref(),
    ];

    if verbose {
        log!("$ cd {:?}", akochan_dir);
        log!(
            "$ {:?}{}",
            akochan_exe,
            args.iter()
                .fold("".to_owned(), |acc, p| format!("{} {:?}", acc, p))
        );
    }

    let mut akochan = Command::new(akochan_exe)
        .args(args)
        .current_dir(Path::new(akochan_dir))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn akochan")?;

    let stdin = akochan
        .stdin
        .as_mut()
        .context("failed to get stdin of akochan")?;
    let mut stdout_lines = BufReader::new(
        akochan
            .stdout
            .as_mut()
            .context("failed to get stdout of akochan")?,
    )
    .lines();

    let events_len = events.len();
    let mut total_reviewed = 0;
    let mut total_tolerated = 0;
    let mut total_problems = 0;
    let mut raw_score = 0.;

    let mut kyoku_review = KyokuReview::default();
    let mut state = State::new(target_actor);
    let mut junme = 0;
    let mut entries = vec![];
    let mut is_reached = false;

    for (i, event) in events.iter().enumerate() {
        let to_write = json::to_string(event).unwrap();
        writeln!(stdin, "{}", to_write).context("failed to write to akochan")?;
        if verbose {
            log!("> {}", to_write);
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
                let kyoku = (bakaze.as_u8() - Pai::East.as_u8()) * 4 + kk - 1;
                kyoku_review.kyoku = kyoku;
                kyoku_review.honba = honba;
                is_reached = false;

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
                if actor == target_actor {
                    continue;
                }
            }

            Event::Tsumo { actor, .. } => {
                if actor != target_actor {
                    continue;
                }
                junme += 1;
            }

            Event::Chi { actor, .. } | Event::Pon { actor, .. } => {
                if actor == target_actor {
                    junme += 1;
                }
                continue;
            }

            Event::ReachAccepted { actor } => {
                if actor == target_actor {
                    is_reached = true;
                }
                continue;
            }

            _ => continue,
        };

        log!(
            "reviewing kyoku={} honba={} junme={} ({:.2}%)",
            kyoku_review.kyoku,
            kyoku_review.honba,
            junme,
            (i as f32) / (events_len as f32) * 100f32,
        );

        // should have at least 4, e.g. dahai -> ryukyoku -> end_kyoku -> end_game
        if events.len() < i + 4 {
            bail!("wrong size of input events, expected to have 4 more");
        }

        // be careful, stdout_lines.next() may block.
        let line = stdout_lines
            .next()
            .context("failed to read from akochan: unexpected EOF")?
            .context("failed to read from akochan")?;
        if verbose {
            log!("< {}", line.trim());
        }

        let actions: Vec<DetailedAction> =
            json::from_str(&line).context("failed to parse JSON output of akochan")?;

        if actions.is_empty() || actions.iter().any(|a| a.moves.is_empty()) {
            log!("WARNING: actions or some moves in actions is empty");
            continue;
        }

        // skip the comparison when
        // 1. it is not our turn and there is no chance to naki
        // 2. our state is reached and only tsumogiri is possible
        if actions.len() == 1 && (is_reached || actions[0].moves[0] == Event::None) {
            continue;
        }

        let expected_action = &actions[0].moves; // best move
        let actual_action = next_action_for_compare(&events[(i + 1)..]);

        let is_equal_or_innocent = compare_action(actual_action, expected_action, target_actor)
            .context("invalid state in event")?;
        let actual_action_strict = next_action_strict(actual_action, target_actor);

        let (move_score, acceptance) = if is_equal_or_innocent {
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
                        "WARNING: unable to find player's action in akochan's return, expected to find: {:?}, list: {:?}",
                        actual_action_strict,
                        actions.iter().map(|a| a.moves.clone()).collect::<Vec<_>>(),
                    );
                    // Skip this situation as it is unclear for akochan, probably not what
                    // those who set --deviation-threshold expect.
                    continue;
                }

                Some(Some(actual_ev)) => {
                    let range = expected_ev - min_ev;
                    let error = expected_ev - actual_ev;
                    let move_score = if range > 0. { 1. - error / range } else { 1. };

                    let dev = expected_ev - actual_ev;
                    if dev <= deviation_threshold {
                        if verbose {
                            log!(
                                "expected_ev - actual_ev <= deviation_threshold ({} - {} = {} < {})",
                                expected_ev,
                                actual_ev,
                                dev,
                                deviation_threshold,
                            );
                        }
                        (move_score, Acceptance::Tolerable) // not acceptable but tolerable
                    } else {
                        (move_score, Acceptance::Disagree) // not acceptable, the threshold is set but the value is lower than it
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
            // Ditto.
            (1., Acceptance::Agree)
        };

        // handle kakan
        let (actor, pai, is_kakan) = match *event {
            Event::Dahai { actor, pai, .. } | Event::Tsumo { actor, pai, .. } => {
                (actor, pai, false)
            }
            Event::Kakan { actor, pai, .. } => (actor, pai, true),
            _ => {
                bail!("invalid state: no actor or pai found, event: {:?}", event)
            }
        };

        match acceptance {
            Acceptance::Disagree => total_problems += 1,
            Acceptance::Tolerable => total_tolerated += 1,
            Acceptance::Agree => (),
        };
        total_reviewed += 1;
        raw_score += move_score;

        let entry = Entry {
            acceptance,
            junme,
            actor,
            pai,
            is_kakan,
            state: state.clone(),
            expected: expected_action.to_vec(),
            actual: actual_action_strict,
            details: actions,
        };
        log!(
            "review entry created: {:?} ({}/{}/{}, {:.03})",
            acceptance,
            total_problems,
            total_tolerated,
            total_reviewed,
            (raw_score / total_reviewed as f64).powf(2.) * 100.,
        );
        if verbose {
            log!("{:?}", entry);
        }

        entries.push(entry);
    }

    let ecode = akochan.wait()?;
    if !ecode.success() {
        if let Some(code) = ecode.code() {
            bail!("non-zero exit code: {}", code);
        } else {
            bail!("non-zero exit code: Process terminated by signal");
        }
    }

    Ok(Review {
        total_problems,
        total_tolerated,
        total_reviewed,
        score: (raw_score / total_reviewed as f64).powf(2.),
        kyokus: kyoku_reviews,
    })
}

fn next_action_for_compare(events: &[Event]) -> &[Event] {
    match events[0] {
        Event::Dora { .. } | Event::ReachAccepted { .. } => next_action_for_compare(&events[1..]),
        Event::Hora { .. } => &events[..3], // considering multiple rons
        Event::Chi { .. } | Event::Pon { .. } | Event::Reach { .. } => &events[..2],
        _ => &events[..1],
    }
}

/// Get actual action from target_actor's perspective, which will handle
/// Event::None and multiple Event::Hora properly.
///
/// `rough_action` must be the return value of `next_action_for_compare`.
fn next_action_strict(rough_action: &[Event], target_actor: u8) -> Vec<Event> {
    match rough_action[0] {
        // passed when it's supposed to naki
        Event::Tsumo { .. } => vec![Event::None],

        // filter the actor's hora from multiple horas
        Event::Hora { .. } => vec![rough_action
            .iter()
            .take(3)
            .find(|&a| matches!(*a, Event::Hora { actor, .. } if actor == target_actor))
            .cloned()
            .unwrap_or(Event::None)],

        _ => match rough_action[0].actor() {
            // not the target actor, who did nothing (passed a possible naki)
            Some(actual_actor) if actual_actor != target_actor => vec![Event::None],

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
    target_actor: u8,
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
            } => Ok(actual_consumed == consumed),
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
                            bail!("event after Reach is not Dahai, got {:?}", next_expected)
                        }
                    } else {
                        bail!("event after Reach is not Dahai, got {:?}", next_actual)
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
                } if actual_consumed == consumed => {
                    let next_actual = actual_action.get(1);

                    if let Some(&Event::Dahai {
                        pai: actual_pai, ..
                    }) = next_actual
                    {
                        let next_expected = expected_action.get(1);

                        if let Some(&Event::Dahai { pai, .. }) = next_expected {
                            Ok(actual_pai == pai)
                        } else {
                            bail!("event after Chi is not Dahai, got {:?}", next_expected)
                        }
                    } else {
                        bail!("event after Chi is not Dahai, got {:?}", next_actual)
                    }
                }
                _ => {
                    if let Some(actor) = actual.actor() {
                        // interrupted by opponent's pon, daiminkan or ron
                        Ok(actor != target_actor)
                    } else {
                        bail!("unexpected event: {:?}", actual)
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
                } if actual_consumed == consumed => {
                    let next_actual = actual_action.get(1);

                    if let Some(&Event::Dahai {
                        pai: actual_pai, ..
                    }) = next_actual
                    {
                        let next_expected = expected_action.get(1);

                        if let Some(&Event::Dahai { pai, .. }) = next_expected {
                            Ok(actual_pai == pai)
                        } else {
                            bail!("event after Pon is not Dahai, got {:?}", next_expected)
                        }
                    } else {
                        bail!("event after Pon is not Dahai, got {:?}", next_actual)
                    }
                }
                _ => {
                    if let Some(actor) = actual.actor() {
                        // interrupted by opponent's ron
                        Ok(actor != target_actor)
                    } else {
                        bail!("unexpected event: {:?}", actual)
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
                        Ok(actor != target_actor)
                    } else {
                        bail!("unexpected event: {:?}", actual)
                    }
                }
            }
        }

        // considering multiple rons
        Event::Hora { .. } => Ok(actual_action.iter().take(3).any(|a| {
            if let Event::Hora { actor, .. } = *a {
                actor == target_actor
            } else {
                false
            }
        })),

        Event::None => match actual {
            // issue #19
            Event::Tsumo { .. } | Event::Ryukyoku { .. } => Ok(true),
            _ => {
                if let Some(actor) = actual.actor() {
                    Ok(actor != target_actor)
                } else {
                    bail!("unexpected event: {:?}", actual)
                }
            }
        },

        Event::Ryukyoku { .. } => match actual {
            Event::Ryukyoku { .. } => Ok(true),
            _ => Ok(false),
        },

        _ => bail!("unexpected event: {:?}", actual),
    }
}
