# Adamantium

> Immutable, shareable, functional data structures in Rust.

These data structures use `Arc` rather than `Box` as their pointer type,
allowing them to support non-blocking concurrent reads and be memory
efficient by sharing non-modified sub-structures.

Currently Implemented:

 - Cons-List
 - Size-Balanced Binary-Search-Tree

Future Plans:
 - Patricia Tree
 - General Trie
 - Heap of some kind
 - Priority Queue of some kind
 - O(1) Deque
 - Suggestions Welcome :)

