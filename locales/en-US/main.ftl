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

end-status-ron = {action-ron} by {$seat}{$delta}
end-status-ryuukyoku = {action-ryuukyoku}
end-status-tsumo = {action-tsumo} by {$seat}{$delta}

kyoku =
    {$bakaze} {$kyoku}{$honba ->
        [0] {""}
        *[other] -{$honba}
    }

turn =
    Turn {$junme}{$junme ->
        [0] {""}
        *[other] (×{$tiles-left})
    }

seat-self = Self👇
seat-shimocha = Shimocha👉
seat-toimen = Toimen👆
seat-kamicha = Kamicha👈
