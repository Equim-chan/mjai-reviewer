action-chii = チー
action-chiicut = チー、打
action-discard = 打
action-kan = カン
action-skip = スルー
action-pon = ポン
action-poncut = ポン、打
action-riichi = 立直
action-ron = ロン
action-ryuukyoku = 流局
action-tsumo = ツモ

donate-header = 寄付

end-status-ron = {$seat}{"\u00a0"}{action-ron} {$delta}
end-status-ryuukyoku = {action-ryuukyoku}
end-status-tsumo = {$seat}{"\u00a0"}{action-tsumo} {$delta}

final-ranking-probs-at-the-start-of-kyoku = {$kyoku}開始時点の最終順位確率

game-summary-header = 目次

help-header = ヘルプ

kyoku =
    {$bakaze ->
        [East] 東
        [South] 南
        [West] 西
        [North] 北
        *[other] {$bakaze}
    }{$kyoku-in-bakaze}局{$honba ->
        [0] {""}
        *[other] {" "}{$honba}本場
    }

metadata-engine-header = AI
metadata-game-length-header = 対局の長さ
metadata-game-length-value = {$length ->
    [Hanchan] 半荘
    [Tonpuu] 東風
    *[other] {$length}
}
metadata-generated-at-header = 生成日時
metadata-header = メタデータ
metadata-loading-time-header = ロード時間
metadata-log-id-header = ログID
metadata-match-rate-header = AI一致率
metadata-mjai-reviewer-version-header = mjai-reviewerバージョン
metadata-player-id-header = プレイヤーID
metadata-review-time-header = 検討時間

panel-expand = 展開:
panel-expand-all = 全て
panel-expand-diff-only = 差分のみ
panel-expand-none = なし
panel-layout = レイアウト:
panel-layout-horizontal = 横向
panel-layout-vertical = 縦向
panel-save-this-page = このページを保存

place-percentage = {$rank}位率 (%)

player = プレイヤー

replay-viewer = 牌譜ビューア

score-header = 点数

tehai-cuts = {$player}打
tehai-draw = 自摸
tehai-kans = {$player}加槓
tehai-riichi = 立直中

tenhou-net-6-json-log-header = tenhou.net/6 JSON ログ

tenhou-net-6-paste-instruction-before-link = {""}
tenhou-net-6-paste-instruction-after-link = {" "}の EDIT AS TEXT に貼り付けられます。

title = 牌譜検討

turn = {$junme}巡目 (残り{$tiles-left})
turn-info-furiten = (振り聴)
turn-info-shanten = {$shanten}向聴
turn-info-tenpai = 聴牌

seat-self = 自家
seat-kamicha = 上家
seat-shimocha = 下家
seat-toimen = 対面
