action-chii = 吃
action-chiicut = 吃、打
action-discard = 打
action-kan = 杠
action-pass = 跳过
action-pon = 碰
action-poncut = 碰、打
action-riichi = 立直
action-ron = 荣
action-ryuukyoku = 流局
action-tsumo = 自摸

donate-header = 打赏

end-status-ron = {$seat}{"\u00a0"}{action-ron} {$delta}
end-status-ryuukyoku = {action-ryuukyoku}
end-status-tsumo = {$seat}{"\u00a0"}{action-tsumo} {$delta}

final-ranking-probs-at-the-start-of-kyoku = {$kyoku}开局时的最终顺位概率

game-summary-header = 目录

help-header = 帮助

kyoku =
    {$bakaze ->
        [East] 东
        [South] 南
        [West] 西
        [North] 北
        *[other] {$bakaze}
    }{$kyoku-in-bakaze}局{$honba ->
        [0] {""}
        *[other] {$honba}本场
    }

metadata-engine-header = 引擎
metadata-game-length-header = 对局长度
metadata-game-length-value = {$length ->
    [Hanchan] 半庄
    [Tonpuu] 东风
    *[other] {$length}
}
metadata-generated-at-header = 生成时间
metadata-header = 元数据
metadata-loading-time-header = 载入用时
metadata-log-id-header = 牌谱 ID
metadata-match-rate-header = AI 一致率
metadata-mjai-reviewer-version-header = mjai-reviewer 版本
metadata-player-id-header = 玩家 ID
metadata-review-time-header = 检讨用时

panel-expand = 展开:
panel-expand-all = 全部
panel-expand-diff-only = 仅差异项
panel-expand-none = 无
panel-layout = 布局:
panel-layout-horizontal = 水平
panel-layout-vertical = 垂直
panel-save-this-page = 保存本页面

place-percentage = {$rank}位率 (%)

player = 玩家

replay-viewer = 牌谱回放

score-header = 点数

tehai-cuts = {$player}打
tehai-draw = 自摸
tehai-kans = {$player}加杠
tehai-riichi = 立直

tenhou-net-6-json-log-header = tenhou.net/6 JSON 牌谱

tenhou-net-6-paste-instruction-before-link = 可粘贴到{" "}
tenhou-net-6-paste-instruction-after-link = {" "}的「EDIT AS TEXT」选项里。

title = 牌谱检讨

turn = {$junme}巡 (余{$tiles-left})
turn-info-furiten = (振听)
turn-info-shanten = {$shanten}向听
turn-info-tenpai = 听牌

seat-kamicha = 上家
seat-self = 自家
seat-shimocha = 下家
seat-toimen = 对家
