# Modeling movements with linear algebra
## Motivation
Some of the most important fundamental concepts of solving a puzzle in *Willo's Graveyard* have to do with the concept of what we have been referring to as range.
Range deals with what positions are reachable for the player based on their current movement table.
Some of the most common ranges have been referred to as:
- Utility range.
The player can pretty much go wherever they want, on a checkerboard.
- Diagonal traps.
The player has access to half the level, but the other half across the diagonal is inaccessable.
- Subgrid range.
The player may be able to go wherever throughout the level, but can only access a subset of that range that lies on a 4x4 lattice.

This concept of range has been difficult to define rigorously.
However, this concept is reminiscent of spans in linear algebra.
If we model movements as vectors, then the movement table can be modeled as a set of movement vectors.
The range of the movement table can then be modeled as the span of the set of movement vectors.

## Movement vectors
Let's define a movement vector.
We'll represent a movement as a 2-dimensional vector.
The first value represents the movement's change in the x-direction in tiles, and the second value represents the movement's change in the y-direction.

#### Examples:
1. the up-right movement would be represented as ${\vec{m} = \begin{pmatrix} 1 \\\\ 1 \end{pmatrix}}$
2. the left-left (aka left-straight) movement would be represented as ${\vec{m} = \begin{pmatrix} -2 \\\\ 0 \end{pmatrix}}$
3. the up-down movement would be represented as ${\vec{m} = \begin{pmatrix} 0 \\\\ 0 \end{pmatrix}}$

Every movement is made up of two sub-movements called components.
The up-right movement is made up of an up component followed by a right component.
The left-left movement is made up of two left components.
The current state of the game only has a basic movement table, without movable arrows.
In this iteration, each component is simply a cardinal direction.
It's worth noting that movable arrows will introduce 0-components and diagonal components.
So, for now, the set of all components ${C}$ is the same as the set of all vectors representing cardinal directions:

$$
C = \\{
\begin{pmatrix} 0 \\\\ 1 \end{pmatrix},
\begin{pmatrix} -1 \\\\ 0 \end{pmatrix},
\begin{pmatrix} 0 \\\\ -1 \end{pmatrix},
\begin{pmatrix} 1 \\\\ 0 \end{pmatrix}
\\}
$$

Then, the set of all possible movement vectors ${M}$ can be defined as:

$$ M = \\{ \vec{c}_1 + \vec{c}_2: \vec{c}_1, \vec{c}_2 \in C \\} $$

Finally, a particular movement table ${T}$ could be thought of as some subset of ${M}$.

### Limitations
This model isn't 100% analogous to the game world.
The game world complicates things by having walls, which can nullify movement components.
The movement vectors also don't express the order of the components, instead only expressing the total change, or "movement delta".

The set of all movements ${M}$ may also be misleading since, due to the uniqueness of set elements, it only contains unique movement deltas.
Meanwhile, in game, for all turn movements there exists a similar turn movement on the table that has the same components in the opposite order.
There's also the issue of cancel moves, despite having different components in different orders, all 4 of them share the same movement delta 0, which only appears in ${M}$ once.
This may complicate things if we ever need to consider the possibility of two moves being similar on a table while also considering the impossibility that those two moves are actually the same.

The purpose of this model is to have a set of vectors we can use to discuss the topic of range.
Again, range seems to be a similar concept to spans.
To avoid hitting these limitations, we can just think about these thinggs in the context of a level without walls, and be considerate when thinking about repeated/similar movements.

However, the analogy between range and spans actually breaks down a little bit with this model.
To define a span, we need to consider the possible linear combinations of a set of vectors.
Linear combinations *do* have an analogy to the game in our model, the vector terms being the movement vectors and their coefficients being the number of times those movements were enacted by the player.
However, with this analogy, the only linear combinations that are valid in our game are ones with natural-number coefficients.
The coefficients cannot be non-integers, since there's no way to perform half a movement delta in the game.
The coefficients also cannot be negative, doing "negative" movements is a bit more useful that what is available to the player.
For example, it's not like having the move right-right means that the player can also perform the move left-left.

So, we need to only consider "natural" linear combinations, only construct "natural" spans, and only do any of this in a level with no walls.

### 3-dimensional model
There was an idea to model the movement vectors in 3 dimensions instead of 2, in order to ease the limitations somewhat.
The 3rd value of every movement vector would simply be 1.
This 3rd value would represent the number of movements that have been performed.

This eases the natural limitation on linear combinations of the movement vectors.
In particular, we don't need to worry about negative coefficients anymore.
For example, negative coefficients on a right-right movement vector would no longer equate to a left-left movement.
After all, the actual left-left movement has a positive 1 in the 3rd dimension, while a negative right-right movement would have a negative one.
Instead, negative right-right would be more like "undoing" a right-right movement, going back in time.
In terms of linear combinations and spans, negative coefficients wouldn't be a problem for the accuracy of the model.

However, there is a new "limitation".
A simple position in the level is now no longer a particular coordinate, but an entire line in 3d space.
The consequence is that some concepts like linear independence don't behave quite like we expect them to.
For example, we may wand a model where the set up-right, right-down, and right-right are *not* linearly independent, since performaing the first two moves results in the same delta as the last.
However, this isn't true in this 3d model, since the sum of the first two moves would have a value of 2 in the third dimension, while the last move would only have a value of 1.
Despite being in the same position in-game, they are not in the same position in the model.

So, either way, there is a trade-off.
For now, I will continue to work with the 2d model.
