# The Mathematics of Cauchy Consensus
## Prefixes
Consider a set *Q*, we denote the set of finite length sequences of elements of *Q* as Seq(*Q*).

A prefix of some element *x* = (*x*<sub>1</sub>, ... ,*x*<sub>n</sub>) in *Q* is a contiguous subsequence of *x* sharing the same first element. Prefixes of *x* take the form (*x*<sub>1</sub>,...,*x*<sub>n'</sub>)* where *n* ≤ *n'*.

For any sequences *x* = (*x*<sub>1</sub>,...,*x*<sub>n</sub>) and *y* = (*y*<sub>1</sub>,...,*y*<sub>m</sub>) we define *x*∧*y* to be the longest common prefix. 

The longest common prefix operation enjoys the following properties:
* *x*∧*y* = *y*∧*x* (commutativity),
* (*x*∧*y*)∧*z* = *x*∧(*y*∧*z*) (associativity),
* *x*∧*x* = x (idempotency), and
* ()∧*x* = () (lower-bounded),

making it a [meet operation](https://en.wikipedia.org/wiki/Join_and_meet).

These properties describe a lower [meet-semilattice structure](https://en.wikipedia.org/wiki/Semilattice) on Seq(*Q*).

The length of the longest common prefix |*x*∧*y*|, denoted <*x*, *y*>, enjoys the following properties:
* <*x*, *y*> = <*y*, *x*>
* <*x*∨*y*, *z*> = <*x*, *y*∨*z*>
* <*x*, *y*> = |*x*|

Using the properties above we define a metric on Seq(*Q*) as follows:
* *d*(*x*, *y*) = |*x*| + |*y*| - 2 <*x*, *y*>

Intuitively the distance *d*(*x*, *y*) is the minimum number of appends + pops required to transition from sequence *x* to sequence *y*.

## Cartesian Products
We can extend this structure to the cartesian product Seq(*Q*)×Seq(*Q*):
* (*x*, *x'*)∧(*y*, *y'*) = (*x*∧*y*, *x'*∧*y'*)
* <(*x*, *x'*), (*y*, *y'*)> = <*x*, *y*> + <*x'*, *y'*>
* *d*((*x*, *x'*), (*y*, *y'*)) = *d*(*x*, *y*) + *d*(*x'*, *y'*)

More generally, if we have a two finite products of sequences indexed by *A*, (*x*<sub>*a*</sub>)<sub>*a* in *A*</sub> and (*y*<sub>*a*</sub>)<sub>*a* in *A*</sub>, then:
* *d*((*x*<sub>*a*</sub>)<sub>*a* in *A*</sub>, (*y*<sub>*a*</sub>)<sub>*a* in *A*</sub>) = Sum over *a* in *A* of d(*x*<sub>*a*</sub>, *y*<sub>*a*</sub>).

Extending the intuition from earlier, one could imagine two collections of sequences, both indexed by some set *A*, where transitions are performed between sequences with matching indexes.

## Homomorphisms and Blockchains
If we have a mapping, *f*: Seq(*Q*) -> *D*, from sequences of *Q* to some set *D* then we may construct a map, *g*:Seq(*Q*) -> 2<sup>*D*</sup>, from sequences of *Q* to subsets of *D*. This can be done as follows:
* *g*((*x*<sub>1</sub>,...,*x*<sub>n</sub>)) = { f(*x*<sub>1</sub>,...,*x*<sub>i</sub>) | 0 < i ≤ n }

This *g* is a homomorphism, preserving the meet semi-lattice structure:
* *g*(*x*∧*y*) = *g*(*x*) ∩ *g*(*y*)

If *f* is injective then so is *g*. An injective *g* additionally preserves distance:
* d(*x*, *y*) = |*x*| + |*y*| - 2 <*x*, *y*> = |*g*(*x*)| + |*g*(*y*)| - 2 |*g*(*x*) ∩ *g*(*y*)| = |*g*(*x*) ⊖ *g*(*y*)| = d(*g*(*x*), *g*(*y*))

Such an *f* is the blockchain structure, where 
* *Q* is the set of ledger states,
* *x* in Seq(*Q*) is a sequence of states (a history),
* *D* is the set of hash digests, and
* *f* takes a sequence of states and maps it injectively to the sequence of chain tips.

For blockchains *d*(*x*, *y*) is equivalent to the number of blocks to be removed then added during a reorg.

Again, *g* can be extended to *g*': Seq(*Q*)×...×Seq(*Q*) -> *D* where *g*' = *g*(*s*<sub>1</sub>||*x*<sub>1</sub>)∪...∪*g*(*s*<sub>1</sub>||*x*<sub>n</sub>) where *s*<sub>i</sub> is some salt unique to the index *i*.

## Oddsketches
An oddsketch O(_) is a cryptographic construction similar to a bloom filter. The difference being that when inserting an element into a bloom filter causes the hash indexed bit to be set to 1 while inserting an element into a oddsketch flips the bit.

Oddsketches are such that the hamming weight of O(*X*) xor O(*Y*) is approximately |*X* ⊖ *Y*| where *X* and *Y* are sets.

## Conclusion
The culmination of the points touched upon above is a rough overview of the proof for the following theorem:

If two systems each have a collection of blockchains indexed by *A*, and *X* and *Y* are the respective sets of all chaintips at each transition then |O(*X*) xor O(*Y*)| is approximately the number of a transitions to reorg between them.
