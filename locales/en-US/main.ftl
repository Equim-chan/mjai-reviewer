action-chii = Chii
action-chiicut = Chii, cut
action-discard = Discard
action-kan = Kan
action-pass = Pass
action-pon = Pon
action-poncut = Pon, cut
action-riichi = Riichi
action-ron = Ron
action-ryuukyoku = Ryuukyoku
action-tsumo = Tsumo

donate-header = Donate

end-status-ron = {action-ron} by{"\u00a0"}{$seat}{$delta}
end-status-ryuukyoku = {action-ryuukyoku}
end-status-tsumo = {action-tsumo} by{"\u00a0"}{$seat}{$delta}

final-ranking-probs-at-the-start-of-kyoku = Final ranking probs at the start of {$kyoku}

game-summary-header = Game Summary

help-header = Help

kyoku =
    {$bakaze} {$kyoku-in-bakaze}{$honba ->
        [0] {""}
        *[other] -{$honba}
    }

metadata-engine-header = engine
metadata-game-length-header = game length
metadata-game-length-value = {$length}
metadata-generated-at-header = generated at
metadata-header = Metadata
metadata-loading-time-header = loading time
metadata-log-id-header = log id
metadata-match-rate-header = matches/total
metadata-mjai-reviewer-version-header = mjai-reviewer version
metadata-player-id-header = player id
metadata-review-time-header = review time

panel-expand = Expand:
panel-expand-all = All
panel-expand-diff-only = Diff only
panel-expand-none = None
panel-layout = Layout:
panel-layout-horizontal = Horizontal
panel-layout-vertical = Vertical
panel-save-this-page = 💾Save this page

place-percentage =
    {$rank ->
        [1] {$rank}st
        [2] {$rank}nd
        [3] {$rank}rd
        *[other] {$rank}th
    } place (%)

player = Player

replay-viewer = Replay viewer

score-header = Score

tehai-cuts = {$player} cuts
tehai-draw = Draw
tehai-kans = {$player} kans
tehai-riichi = and declares riichi

tenhou-net-6-json-log-header = tenhou.net/6 JSON log

tenhou-net-6-paste-instruction-before-link = Paste it in{" "}
tenhou-net-6-paste-instruction-after-link = {" "}- EDIT AS TEXT.

title = Replay Examination

turn =
    Turn {$junme}{$junme ->
        [0] {""}
        *[other] {" "}(×{$tiles-left})
    }

turn-info-furiten = (furiten)
turn-info-shanten = {$shanten} shanten
turn-info-tenpai = tenpai

seat-kamicha = Kamicha👈
seat-self = Self👇
seat-shimocha = Shimocha👉
seat-toimen = Toimen👆
