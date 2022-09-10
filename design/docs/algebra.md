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
We'll represent a movement as a vector in $$\N^2$$.

