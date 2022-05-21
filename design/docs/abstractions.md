# Levels of abstraction in Willos Graveyard
I think it may be useful to frame the design of this game around levels of abstraction.
Certain challenges may become more reasonable when the player has achieved a certain level of abstract thought concerning the gameplay.
Preparing players for these challenges by guiding them through these abstractions should result in a smoother experience with fewer unintentional difficulty spikes.

Layers of abstraction:
1. Positions
2. Movements
3. Tables
4. Table Categories
5. Components

## Positions
This level of abstraction is concerned with, well, positioning.
At the start of the game, it's difficult to just put the ghost on the intended tile.
Players are pretty much just stumbling through the movements as beginners
At an intermediate level, players begin noticing some patterns regarding positioning range.

### Beginner
- How to use their current movement set to get from point A to B
- How to position yourself to push a stone in a particular direction
- How to use their current movement set to push stones from point A to B

### Intermediate
- What is the range of positions available without walls
- How to use walls to increase range
  - How to "climb diagonals"
  - How to "change subgrid"
  - How to "change color"
- What is the range of positions available even with walls


## Movements
At this level of abstraction, players can think about what tile on the table they need to solve a problem.
Players at this level find themselves in traps a lot.
At an intermediate level, players start identifying tile types and traps.
They also learn an important lesson in redundancy.

### Beginner
- What movement is required to navigate an obstacle
- What current movement is unnecessary for navigating an obstacle

### Intermediate
- Identifying tile types and their symmetries: 
  - What tiles are "cancels"
  - What tiles are "turns"
  - What tiles are "straights
- What change is required to escape a "diagonal trap" (and maybe accidentally enter another one)
- What change is required to make some stone redundant

## Tables
At this level of abstraction, players can think about the entire table, and what configuration they need to solve a problem.

### Beginner
- What table allows you to navigate some obstacle
- What order do changes need to be made in to preserve redundancy

### Intermediate
- How to make a "wide" change while avoiding traps
- What change is required to make a "utility table"
- Aka, what change is required to remove all diagonal traps
- How to make a change and enter an ideal position afterward (especially if that change creates a trap)


## Table Categories
At this level of abstraction, players have noticed some categories of table and can think about what *kind* of table they need to solve a problem.

Some examples of table categories:
- Utility tables (allow near-complete freedom of movement)
  - Double straights
  - Double turns
    - Train utility tables (aka kneels)
    - Forks and Ts (very intuitive)
- Diagonal traps
  - Single diagonal traps (trap you in a corner)
  - Double diagonal traps
    - Edge traps (lock you out of a direction entirely)
    - Exact diagonal traps (trap you along the exact diagonal of the level)

### Beginner
- What "utility table" meets some criteria
  - What table allows "train movement" in a particular direction


### Intermediate
- What table patterns create "diagonal traps" in general
- What table patterns create "utility tables" in general
- What table patterns allow "train movement" in general

Are there any situations where a diagonal trap is preferred over a utility table?



# Analysis of current levels in terms of abstraction

## Who put this gravestone here?
Pre-positional, just introducing sokoban mechanics

## It takes some getting used to.
Beginner Positional

## Set the table.
Mostly positional.
You are required to make a movement change here but it currently doesn't ask you to think about it too much.
But even then, the positional challenge here isn't that great either, the previous level is harder.

## Exorcism?!
Beginner Movement.
I'm not 100% sure that the positional challenge here is prepared enough by previous levels.
I'm also not convinced that the movement challenge here is designed enough.

## The bishop changes colors
Intermediate Positional?
Is it okay that we're going back one layer of abstraction here?
Reinforcement can be good, plus the concept here is more advanced positional.
This level is obviously meant to teach one of the most important positional ideas, "colors", and I think it does so pretty well.
Maybe it should go earlier? But exorcism was intended to teach about death tiles.

## Full Control
Intermediate Movement
There is a movement lesson here, trying to drive home redundancy.
Is this designed enough?

## Yoink
Beginner Table?
I still wonder if this level is prepared enough.

## Origami fortune teller
Intermediate Positional/Beginner Table
Finally we're driving home the concept of diagonal traps.

## All tied up
Intermediate Movement

## Sokoban, sokoban
Beginner Table

## Training week
Beginner Table Categories

## Two Zs
Intermediate Table

## Don't worry, I'll put it back
Beginner Table

## Martyr
Intermediate Movement?

## A simple variation
Beginner Table Categories

## No stone unturned
Intermediate Table
