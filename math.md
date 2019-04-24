# The Mathematics of Cauchy Consensus
## Prefixes
Consider a set *Q*, we denote the set of finite length sequences of elements of *Q* as Seq(*Q*).

A prefix of some element *x* = (*x*<sub>1</sub>, ... ,*x*<sub>n</sub>) in *Q* is a contiguous subsequence of *x* sharing the same first element. Prefixes of *x* take the form (*x<sub>1</sub>,...,*x*<sub>n'</sub>)* where *n* ≤ *n'*.

For any sequences *x* = (*x*<sub>1</sub>,...,*x*<sub>n</sub>) and *y* = (*y*<sub>1</sub>,...,*y*<sub>m</sub>) we define *x*∨*y* to be the longest common prefix. 

The longest common prefix operation enjoys the following properties:
* *x*∨*y* = *y*∨*x* (commutativity)
* (*x*∨*y*)∨*z* = *x*∨(*y*∨*z*) (associativity)
* *x*∨*x* = x (idempotency)
* ()∨*x* = () (lower-bounded)

These properties describe a lower-semilattice structure on Seq(*Q*).

The length of the longest common prefix |*x*∨*y*|, denoted <*x*, *y*>, enjoys the following properties:
* <*x*, *y*> = <*y*, *x*>
* <*x*∨*y*, *z*> = <*x*, *y*∨*z*>
* <*x*, *y*> = |x|

Using the properties above we define a metric on Seq(*Q*) as follows:
* *d*(*x*, *y*) = |x| + |y| - 2 <*x*, *y*>

Intuitively the distance *d*(*x*, *y*) is the minimum number of appends + pops required to transition from sequence *x* to sequence *y*.

## Cartesian Products
We can extend this structure to the cartesian product:
* (*x*, *x'*)∨(*y*, *y'*) = (*x*∨*y*, *x'*∨*y'*)
* <(*x*, *x'*), (*y*, *y'*)> = <*x*, *y*> + <*x'*, *y'*>
* *d*((*x*, *x'*), (*y*, *y'*)) = *d*(*x*, *y*) + *d*(*x'*, *y'*)

More generally, if we have a products, indexed by *A*, (*x*<sub>*a*</sub>)<sub>*a* in *A*</sub> and (*y*<sub>*a*</sub>)<sub>*a* in *A*</sub> then:
* *d*((*x*<sub>*a*</sub>)<sub>*a* in *A*</sub>, (*y*<sub>*a*</sub>)<sub>*a* in *A*</sub>) = Sum over *a* in *A* of d(*x*<sub>*a*</sub>, *y*<sub>*a*</sub>).
