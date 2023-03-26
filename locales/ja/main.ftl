action-chii = チー
action-chiicut = チー、打
action-discard = 打
action-kan = カン
action-pon = ポン
action-poncut = ポン、打
action-riichi = 立直
action-ron = ロン
action-ryuukyoku = 流局
action-tsumo = ツモ

end-status-ron = {action-ron} {$seat} {$delta}
end-status-ryuukyoku = {action-ryuukyoku}
end-status-tsumo = {action-tsumo} {$seat} {$delta}

kyoku =
    {$bakaze ->
        [East] 東
        [South] 南
        [West] 西
        *[North] 北
    }{$kyoku}局{$honba ->
        [0] {""}
        *[other] {$honba}本場
    }

turn =
    {$junme}巡目 {$junme ->
        [0] {""}
        *[other] (残り{$tiles-left})
    }

seat-self = 自家
seat-shimocha = 下家
seat-toimen = 対面
seat-kamicha = 上家
