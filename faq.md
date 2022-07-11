# FAQ
## Can it review three-player mahjong log?
No, it can't. Three-player mahjong is a completely different game.

## What is pt?
pt refers to the same concept of Tenhou ranking pt. Simply put, they are the weighted version of final placements at the end of the game. `90,45,0,-135` is the pt distribution for a 7 dan player in Tenhou houou hanchan.

## How good are the engines in mahjong?
I have **absolutely no idea** on how strong Mortal or akochan is in general human criteria such as "xxx dan on Tenhou". The reason is obvious and simple: Tenhou (and Mahjong Soul too) probably won't allow an AI that is developed by an individual, to play on their platforms in ranked lobbies.

If you really want to know the answer, you could ask Tenhou or Mahjong Soul officials to let them permit individual developed AIs (preferably Mortal) to legitimately play on their ranked lobbies.

## How good are the engines compared against each other?
In duplicate mahjong, 1 akochan vs 3 Mortal setting and `90,45,0,-135` pt scale for measure, Mortal outplays `90,45,0,-135`-akochan with average rank 2.479 and average pt 1.677, and outplays `90,30,-30,-90`-akochan with average rank 2.482 and average pt 1.961.

Details about this can be found in [Mortal's documentation](https://mortal.ekyu.moe/perf/strength.html#mortal-vs-akochan).

## (Mortal) Where is the deal-in rate column?
If you're referring to the deal-in rate column in akochan, Mortal does not have it; in fact, it was never explicitly calculated by Mortal in the first place. Mortal and akochan are two entirely different mahjong AI engines, created by different developers with different designs. So you probably shouldn't expect them to share any features. 

## (Mortal) What do the notations mean?
$P_k^p$ is a vector that consists of 4 possibility values for player $p$ to achieve the 4 corresponding placements, estimated at the start of kyoku $k$ in this game.

$\Phi_k$ is the pt EV, estimated at the start of kyoku $k$ in this game.

$\hat Q^\pi(s_k^i, a_k^i)$ is the [Q values](https://en.wikipedia.org/wiki/Q-learning) evaluated by the model,
at the $i$-th state of kyoku $k$ in this game,
with $\pi$ representing the policy of the model.

The target of Q value optimization for Mortal is $\Phi_{k+1} - \Phi_k$,
so theoretically $\hat Q^\pi(s_k^i, a_k^i) + \Phi_k$ is an estimation to the pt EV.

## (Mortal) Why do all actions except the best sometimes have significantly lower Q values than that of the best?
As mentioned above, $\hat Q^\pi(s_k^i, a_k^i) + \Phi_k$ is an estimation to the pt EV. However, the evaluation for this value is **<ins>the means but not the objective</ins>**. To be clear, the real fundamental objective for Mortal as a mahjong AI is to achieve the best performance in a mahjong game, but not to calculate accurate scores for all actions. As a result, the evaluated values of all actions but the best may be inaccurate; they only serve as a means to determine its preference for exploration in training.

This is an exploitation vs exploration dilemma. To begin with, Mortal is [model-free](https://en.wikipedia.org/wiki/Model-free_(reinforcement_learning)), which means it cannot obtain the optimize target, the actual Q values $Q^\pi(s_k^i, a_k^i)$, without actually evaluate the action $a$.
Therefore, if we intend to make actions' Q values more accurate, the model will have to explore those less likely actions more, which may lead to overestimation on some bad actions, making it performs worse. In a game with so much randomness like mahjong, such overestimation is very likely to happen since the variance is very high. To avoid such performance regression, the model needs to exploit more, leading to less accurate predicted Q values $\hat Q^\pi(s_k^i, a_k^i)$.

ELI5: **<ins>Mortal is optimized for playing, not reviewing or reasoning.</ins>**

## (akochan) How to configure the pt distribution?
In `tactics.json`, change `jun_pt` value. Note that there is a hard-coded bound of $[-200, 200]$ for every element.

## (akochan) Why does akochan act so weird sometimes?
Akochan is not good at kan. Akochan also has numerical stability issues in extreme situations.

Akochan is very aggressive about its sole goal - the "final" pt EV, instead of just winning this round.

## How is the rating calculated?
$$
100 \times (
    \frac{1}{k} \displaystyle \sum_{k=1}^k
    \frac{1}{i} \displaystyle \sum_{i=1}^n
    \frac
    {\hat Q^\pi(s_k^i, a_k^i) - \displaystyle \min_a \hat Q^\pi(s_k^i, a_k^i)}
    {\displaystyle \max_a \hat Q^\pi(s_k^i, a_k^i) - \displaystyle \min_a \hat Q^\pi(s_k^i, a_k^i)}
) ^ 2
$$

The calculation is very basic and it is not a reliable measure.
