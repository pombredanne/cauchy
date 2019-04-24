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

making it a meet operation.

These properties describe a lower meet-semilattice structure on Seq(*Q*).

The length of the longest common prefix |*x*∧*y*|, denoted <*x*, *y*>, enjoys the following properties:
* <*x*, *y*> = <*y*, *x*>
* <*x*∨*y*, *z*> = <*x*, *y*∨*z*>
* <*x*, *y*> = |x|

Using the properties above we define a metric on Seq(*Q*) as follows:
* *d*(*x*, *y*) = |x| + |y| - 2 <*x*, *y*>

Intuitively the distance *d*(*x*, *y*) is the minimum number of appends + pops required to transition from sequence *x* to sequence *y*.

## Cartesian Products
We can extend this structure to the cartesian product:
* (*x*, *x'*)∧(*y*, *y'*) = (*x*∧*y*, *x'*∧*y'*)
* <(*x*, *x'*), (*y*, *y'*)> = <*x*, *y*> + <*x'*, *y'*>
* *d*((*x*, *x'*), (*y*, *y'*)) = *d*(*x*, *y*) + *d*(*x'*, *y'*)

More generally, if we have a finite products of sequences, indexed by *A*, (*s*<sub>*a*</sub>)<sub>*a* in *A*</sub> and (*t*<sub>*a*</sub>)<sub>*a* in *A*</sub> then:
* *d*((*s*<sub>*a*</sub>)<sub>*a* in *A*</sub>, (*t*<sub>*a*</sub>)<sub>*a* in *A*</sub>) = Sum over *a* in *A* of d(*s*<sub>*a*</sub>, *t*<sub>*a*</sub>).

Extending the intuition from earlier, one could imagine two collections of sequences, both indexed by some set *A*, where transitions are performed between sequences with matching indexes.

## Homomorphisms and Blockchains
If we have a mapping, *f*: Seq(*Q*) -> *D*, from sequences of *Q* to some set *D* then we may construct a map, *g*:Seq(*Q*) -> 2<sup>*D*</sup>, from sequences of *Q* to subsets of *D*. This can be done as follows:
* *g*((*x*<sub>1</sub>,...,*x*<sub>n</sub>)) = { f(*x*<sub>1</sub>,...,*x*<sub>i</sub>) | 0 < i ≤ n }

This *g* is a homomorphism, preserving the meet semi-lattice structure:
* *g*(*x*∧*y*) = *g*(*x*) ∩ *g*(*y*)

If *f* is injective then so is *g*. An injective *g* additionally preserves distance:
* d(*x*, *y*) = |*x*| + |*y*| - 2 <*x*, *y*> = |*g*(*x*)| + |*g*(*y*)| - 2 |*g*(*x*) ∩ *g*(*y*)| = |*g*(*x*) ⊖ *g*(*y*)| = d(*g*(*x*), *g*(*y*))

Again, this can be extended to the product of sequences.
