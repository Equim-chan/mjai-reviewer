use crate::log;
use anyhow::anyhow;
use anyhow::{Context, Result};
use convlog::mjai::Event;
use convlog::Pai;
use serde::Serialize;
use serde_json;
use std::ffi::OsStr;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Default, Serialize)]
pub struct KyokuReview {
    pub kyoku: u8, // in tenhou.net/6 format, counts from 0
    pub honba: u8,

    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub junme: u8,
    pub actor: u8,
    pub pai: Pai,

    pub expected: Vec<Event>, // at most 2 events
    pub actual: Vec<Event>,   // at most 2 events
}

pub fn review<O, P>(
    akochan_exe: O,
    akochan_dir: P,
    tactics_config_path: O,
    is_full: bool,
    events: &[Event],
    target_actor: u8,
) -> Result<Vec<KyokuReview>>
where
    O: AsRef<OsStr>,
    P: AsRef<Path>,
{
    let mut kyoku_reviews = vec![];

    let target_actor_string = target_actor.to_string();
    let args = &[
        "pipe".as_ref(),
        tactics_config_path.as_ref(),
        target_actor_string.as_ref(),
    ];

    log!("$ cd {:?}", akochan_dir.as_ref());
    log!(
        "$ {:?}{}",
        akochan_exe.as_ref(),
        args.iter()
            .fold("".to_owned(), |acc, p| format!("{} {:?}", acc, p))
    );

    let mut akochan = Command::new(akochan_exe)
        .args(args)
        .current_dir(akochan_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn akochan")?;

    let stdin = akochan
        .stdin
        .as_mut()
        .context("failed to get stdin of akochan")?;
    let mut stdout = BufReader::new(
        akochan
            .stdout
            .as_mut()
            .context("failed to get stdout of akochan")?,
    )
    .lines();

    let mut junme = 0;
    let mut kyoku_review = KyokuReview::default();
    let mut entries = vec![];

    for (i, event) in events.iter().enumerate() {
        let to_write = serde_json::to_string(event).unwrap() + "\n";
        stdin
            .write_all(to_write.as_bytes())
            .context("failed to write to akochan")?;
        log!("> {}", to_write.trim());

        // this match does two things:
        // 1. setting board metadata like bakaze, kyoku, honba, junme
        // 2. decide whether or not this event is a valid timing when we can review
        match *event {
            Event::StartKyoku {
                bakaze,
                kyoku: kk,
                honba: hb,
                ..
            } => {
                kyoku_review.kyoku = (bakaze.0 - 41) * 4 + kk - 1;
                kyoku_review.honba = hb;
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

            Event::Dahai { actor, .. } => {
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

            _ => continue,
        };

        // should have at least 4, e.g. dahai -> ryukyoku -> end_kyoku -> end_game
        if events.len() < i + 4 {
            return Err(anyhow!(
                "wrong size of input events, expected to have 4 more"
            ));
        }

        // be careful, stdout.next() may block.
        let line = stdout
            .next()
            .context("failed to read from akochan: unexpected EOF")?
            .context("failed to read from akochan")?;
        log!("< {}", line.trim());

        let expected: Event =
            serde_json::from_str(&line).context("failed to parse JSON output of akochan")?;

        let expected_action =
            if let Event::Chi { .. } | Event::Pon { .. } | Event::Reach { .. } = expected {
                let next_line = stdout
                    .next()
                    .context("failed to read from akochan: unexpected EOF")?
                    .context("failed to read from akochan")?;
                let next_expected: Event = serde_json::from_str(&next_line)
                    .context("failed to parse JSON output of akochan")?;

                vec![expected, next_expected]
            } else {
                vec![expected]
            };

        let actual_action = next_action_roughly(&events[(i + 1)..]);
        if !is_full && compare_action(&actual_action, &expected_action, target_actor) {
            continue;
        }

        // TODO: properly filter [None]:[None] in full mode
        let actual_action_vec = next_action_exact(actual_action, target_actor);
        let (actor, pai) = match *event {
            Event::Dahai { actor, pai, .. } | Event::Tsumo { actor, pai, .. } => (actor, pai),
            _ => {
                return Err(anyhow!(
                    "invalid state: no actor or pai found, event: {:?}",
                    event
                ))
            }
        };

        let entry = Entry {
            junme,
            actor,
            pai,
            expected: expected_action,
            actual: actual_action_vec,
        };
        log!("created review entry: {:?}", entry);
        entries.push(entry);
    }

    let ecode = akochan.wait()?;
    if !ecode.success() {
        if let Some(code) = ecode.code() {
            return Err(anyhow!("non-zero exit code: {}", code));
        } else {
            return Err(anyhow!("non-zero exit code: Process terminated by signal"));
        }
    }

    Ok(kyoku_reviews)
}

fn next_action_roughly(events: &[Event]) -> &[Event] {
    match events[0] {
        Event::Dora { .. } => next_action_roughly(&events[1..]),
        Event::Hora { .. } => &events[..3], // considering multiple rons
        Event::Chi { .. } | Event::Pon { .. } | Event::Reach { .. } => &events[..2],
        _ => &events[..1],
    }
}

/// Get actual action from target_actor's perspective
fn next_action_exact(rough_action: &[Event], target_actor: u8) -> Vec<Event> {
    match rough_action[0] {
        // passed when it's supposed to naki
        Event::Tsumo { actor, .. } if actor == target_actor => vec![Event::None],

        Event::Hora { .. } => vec![rough_action
            .iter()
            .take(3)
            .find(|&a| {
                if let Event::Hora { actor, .. } = *a {
                    actor == target_actor
                } else {
                    false
                }
            })
            .cloned()
            .unwrap_or(Event::None)],

        _ => match rough_action[0].actor() {
            // not the target actor, who did nothing (usually passed)
            Some(actual_actor) if actual_actor != target_actor => vec![Event::None],

            // anything else the target actor did
            _ => rough_action.to_vec(),
        },
    }
}

/// Returns true if actual_action is innocent or the same as expected_action.
fn compare_action(actual_action: &[Event], expected_action: &[Event], target_actor: u8) -> bool {
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
        return true;
    }

    // fallback to slow path.
    let actual = &actual_action[0];

    match expected_action[0] {
        Event::Dahai { pai, .. } => {
            match *actual {
                // ignore 九種九牌
                Event::Ryukyoku => true,

                // ignore the difference of tsumogiri
                Event::Dahai {
                    pai: actual_pai, ..
                } => actual_pai == pai,

                _ => false,
            }
        }

        Event::Ankan { consumed, .. } => {
            match *actual {
                // ignore 九種九牌
                Event::Ryukyoku => true,

                Event::Ankan {
                    consumed: actual_consumed,
                    ..
                } => actual_consumed == consumed,

                _ => false,
            }
        }

        Event::Kakan { pai, .. } => match *actual {
            Event::Kakan {
                pai: actual_pai, ..
            } => actual_pai == pai,

            _ => false,
        },

        Event::Reach { .. } => {
            match actual {
                Event::Reach { .. } => {
                    // ignore the difference of tsumogiri
                    if let Some(&Event::Dahai {
                        pai: actual_pai, ..
                    }) = actual_action.get(1)
                    {
                        if let Event::Dahai { pai, .. } = expected_action[1] {
                            actual_pai == pai
                        } else {
                            false // ?
                        }
                    } else {
                        false // ?
                    }
                }

                _ => false,
            }
        }

        Event::Chi { consumed, .. } => {
            let naki_part_matches = match *actual {
                // ignore 九種九牌
                Event::Ryukyoku => true,

                Event::Tsumo { .. } => false,

                Event::Chi {
                    consumed: actual_consumed,
                    ..
                } => actual_consumed == consumed,

                _ => {
                    if let Some(actor) = actual.actor() {
                        // pon, daiminkan, ron
                        actor != target_actor
                    } else {
                        false // ?
                    }
                }
            };

            if !naki_part_matches {
                false
            } else if let Event::Dahai {
                pai: actual_pai, ..
            } = actual_action[1]
            {
                if let Event::Dahai { pai, .. } = expected_action[1] {
                    actual_pai == pai
                } else {
                    false // ?
                }
            } else {
                false // ?
            }
        }

        Event::Pon { consumed, .. } => {
            let naki_part_matches = match *actual {
                // ignore 九種九牌
                Event::Ryukyoku => true,

                Event::Tsumo { .. } => false,

                Event::Pon {
                    consumed: actual_consumed,
                    ..
                } => actual_consumed == consumed,

                _ => {
                    if let Some(actor) = actual.actor() {
                        // ron
                        actor != target_actor
                    } else {
                        false // ?
                    }
                }
            };

            if !naki_part_matches {
                false
            } else if let Event::Dahai {
                pai: actual_pai, ..
            } = actual_action[1]
            {
                if let Event::Dahai { pai, .. } = expected_action[1] {
                    actual_pai == pai
                } else {
                    false // ?
                }
            } else {
                false // ?
            }
        }

        Event::Daiminkan { .. } => {
            match actual {
                // ignore 九種九牌
                Event::Ryukyoku => true,

                Event::Tsumo { .. } => false,

                Event::Daiminkan { .. } => true,

                _ => {
                    if let Some(actor) = actual.actor() {
                        actor != target_actor
                    } else {
                        false // ?
                    }
                }
            }
        }

        // considering multiple rons
        Event::Hora { .. } => actual_action.iter().take(3).any(|a| {
            if let Event::Hora { actor, .. } = *a {
                actor == target_actor
            } else {
                false
            }
        }),

        Event::None => {
            match actual {
                // ignore 九種九牌
                Event::Ryukyoku => true,

                Event::Tsumo { .. } => true,

                _ => {
                    if let Some(actor) = actual.actor() {
                        actor != target_actor
                    } else {
                        false // ?
                    }
                }
            }
        }

        _ => false, // ?
    }
}
