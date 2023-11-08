# Synchronization algorithm

## Basic notation

Synchronization is performed using centralized scenario, i.e. there exists a single remote
storage $r$ and $n$ clients $c_j$, $j = \overline{1, n}$. 

Let $t_j$ denote a point in time, when client $c_j$ was synchronized for the last time, 
$t_j \geq 0$ for every $j$. Equality $t_j = 0$ means, that $c_j$ has not been synchronized 
yet.

Let $s_j^t$ be a state of client $c_j$ at time $t$, i.e. a tuple of active and removed
transactions $T$, plans $P$, categories $C$ and accounts $A$ sets: 
${s_j = \left(T, P, C, A\right)}$, where $X = \left(X_{a}, X_{r}\right)$ for $X \in \{T, P, C, A\}$, and $X_{a}$ is a set of active items, $X_{r}$ is a set of removed items.

Let $d_j^{t_1, t_2}$ be a difference between states $s_j^{t_1}$ and $s_j^{t_2}$.

Let $p$ be a password, used for synchronization.


