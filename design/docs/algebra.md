# Modeling movements with linear algebra
## Motivation
Some of the most important fundamental concepts of solving a puzzle in *Willo's Graveyard* have to do with the concept of what we have been referring to as range.
Range deals with what positions are reachable for the player based on their current movement table.
Some of the most common ranges have been referred to as:
- Utility range.
The player has can pretty much go wherever they want, on a checkerboard.
- Diagonal traps.
The player has access to half the level, but the other half across the diagonal is inaccessable.
- Subgrid range.
The player may be able to go wherever throughout the level, but can only access a subset of that range that lies on a 4x4 lattice.

This concept of range has been difficult to define rigorously.
However, this concept is reminiscent of spans in linear algebra.
If we model movements as vectors, then the movement table can be modeled as a set of movement vectors.
The range of the movement table can then be modeled as the span of the set of movement vectors.

## Definitions
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
