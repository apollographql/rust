The cache directory is optional. If present within the `.atlas` dir, the compiler will read from it before hitting the system cache in `~/.atlas`. 

This lets projects include bootstrapping data for non-standard atlases. It also enables easy sharing of atlases which do not have a server anywhere. (This is an important mode, at least for the moment, because no atlas server exists).

The contents of this directory will be managed by cacache.

## Cache structure

TK, but generally:

The cache contains the current state for each atlas under the key `@name-of-atlas`. The state will reference other entries, which will be stored keyed by their content hash. In principle, this process continues all the way up to the axiom—the first state of a given atlas. In practice, not all intermediate states have to be stored on disk. (In particular, states may be elided when a root-signed or user-signed lemma aggregates over them, asserting that the mutations expressed in the lemma have been captured in an aggregate proof.)