# FAQ
## Can it review three-player mahjong log?
No, it can't. Three-player mahjong is a completely different game.

## What is pt?
pt refers to the same concept of Tenhou ranking pt. Simply put, they are the weighted version of final placements at the end of the game. $[90,45,0,-135]$ is the pt distribution for a 7 dan player in Tenhou houou hanchan.

## How good are the engines in mahjong?
I have **absolutely no idea** on how strong Mortal or akochan is in general human criteria such as "xxx dan on Tenhou". The reason is obvious and simple: Tenhou (and Mahjong Soul too) probably won't allow an AI that is developed by an individual, to play on their platforms in ranked lobbies.

If you really want to know the answer, you could ask Tenhou or Mahjong Soul officials to let them permit individual developed AIs (preferably Mortal) to legitimately play in their ranked lobbies.

### But I have seen some accounts claim to be Mortal/akochan on some online platforms?
I have no affiliation to them. I am not running any AI in ranked lobbies and will not do so until an official permission is granted.

## How good are the engines compared against each other?
In duplicate mahjong, 1 akochan vs 3 Mortal setting and $[90,45,0,-135]$ pt scale for measure, Mortal outplays $[90,45,0,-135]$-akochan with average rank 2.479 and average pt 1.677, and outplays $[90,30,-30,-90]$-akochan with average rank 2.484 and average pt 1.827.

Although Mortal's win against akochan may not appear to be too much at first glance, it should be noted that there are three Mortal instances in each game, not just one, so the theoretical best average rank Mortal can achieve is 2, not 1.

Details about this can be found in [Mortal's documentation](https://mortal.ekyu.moe/perf/strength.html#mortal-vs-akochan).

## (Mortal) Where is the deal-in rate column?
If you're referring to the deal-in rate column in akochan, Mortal does not have it; in fact, it was never explicitly calculated by Mortal in the first place. Mortal and akochan are two entirely different mahjong AI engines, created by different developers with different designs. So you probably shouldn't expect them to share any features.

## (Mortal) What do the notations mean?
$P_k^p$ is a vector that consists of 4 possibility values for player $p$ to achieve the 4 corresponding placements, estimated at the start of kyoku $k$ in this game.

$\Phi_k$ is the pt EV, estimated at the start of kyoku $k$ in this game.

$\hat Q^\pi(s, a)$ is the [Q values](https://en.wikipedia.org/wiki/Q-learning) evaluated by the model,
at the $i$-th state of kyoku $k$ in this game,
with $\pi$ representing the policy of the model. The target of Q value optimization for Mortal is $\Phi_{k+1} - \Phi_k$,
so theoretically $\hat Q^\pi(s, a) + \Phi_k$ is an estimation to the pt EV.

$\pi_\tau(a|s)$, in simple terms, can be thought of something similar to the heights of the bar on each tile in hand in NAGA's reviewer, though technically Mortal differs from NAGA in terms of how this distribution is produced and used. It is the output of a softmax function which takes categorical Q values produced by Mortal and then transforms them into a discrete probability distribution, calculated as

$$\pi_\tau(a|s) = \frac{\exp(\hat Q^\pi(s, a) / \tau)}{\sum_i \exp(\hat Q^\pi(s, a_i) / \tau)}$$
where $\tau$ is a parameter representing temperature.

Wrapping up, $\hat Q^\pi(s, a)$ is only for advanced users, because it can be very misleading if the user does not understand the subtle details of how Mortal works under the hood. I have been considering whether to just remove the column or not, but in the end I decided to keep it as is. <ins>Just look up $\pi_\tau(a|s)$ as it is easier to understand.</ins>

## (Mortal) Why do all actions except the best sometimes have significantly lower Q values than that of the best?
As mentioned above, $\hat Q^\pi(s, a) + \Phi_k$ is an estimation to the pt EV. However, the evaluation for this value is <ins>the means but not the objective</ins>. To be clear, the real fundamental objective for Mortal as a mahjong AI is to achieve the best performance in a mahjong game, but not to calculate accurate scores for all actions. As a result, the evaluated values of all actions but the best may be inaccurate; they only serve as a means to determine its preference for exploration in training.

This is an exploitation vs exploration dilemma. To begin with, Mortal is [model-free](https://en.wikipedia.org/wiki/Model-free_(reinforcement_learning)), which means it cannot obtain the optimize target, the actual Q values $Q^\pi(s, a)$, without actually evaluate the action $a$.
Therefore, if we intend to make actions' Q values more accurate, the model will have to explore those less likely actions more, which may lead to overestimation on some bad actions, making it performs worse. In a game with so much randomness like mahjong, such overestimation is very likely to happen since the variance is very high. To avoid such performance regression, the model needs to exploit more, leading to less accurate predicted Q values $\hat Q^\pi(s, a)$.

ELI5: <ins>Mortal is optimized for playing, not reviewing or reasoning.</ins>

## (akochan) How to configure the pt distribution?
Edit `jun_pt` in `tactics.json`. Note that there is a hard-coded bound of $[-200, 200]$ for every element.

## (akochan) Why does akochan act so weird sometimes?
Akochan is not good at kan. Akochan also has numerical stability issues in extreme situations.

Akochan is very aggressive about its sole goal - the "final" pt EV, instead of just winning this round.

## How is the rating calculated?
$$
100 \times (
    \frac{1}{K} \displaystyle \sum_{i=1}^K
    \frac{1}{N_i} \displaystyle \sum_{j=1}^{N_i}
    \frac
    {\hat Q^\pi(s_{i,j}, a_{i,j}) - \displaystyle \min_a \hat Q^\pi(s_{i,j}, a_{i,j})}
    {\displaystyle \max_a \hat Q^\pi(s_{i,j}, a_{i,j}) - \displaystyle \min_a \hat Q^\pi(s_{i,j}, a_{i,j})}
) ^ 2
$$

where $K$ is the number of rounds and $N_i$ is the number of player's actions in $i$-th round.

The calculation is very basic and the result has a high variance. It probably shouldn't be considered a reliable measure.
