# Algo — Rust DSA & LeetCode Interview Prep Library

> A personal study tool for blockchain/systems-engineering technical interviews.
> Every implementation includes **rich, educational comments** explaining *why* the design works, time/space complexity, invariants, and Rust-specific idioms.
> This project is heavily developed using LLMs

## 📁 Module Index

### `dsa/` — Data Structures & Algorithms

#### Trees (`dsa::trees`)

| Module | Data Structure | Key Feature |
|--------|---------------|-------------|
| [`binary_tree`](src/dsa/trees/binary_tree.rs) | Binary Search Tree | Foundation BST with insert/delete/traversal |
| [`avl_tree`](src/dsa/trees/avl_tree.rs) | AVL Tree | Self-balancing via rotations, O(log n) guaranteed |
| [`red_black_tree`](src/dsa/trees/red_black_tree.rs) | Left-Leaning Red-Black Tree | Color invariants, maps to 2-3 trees |
| [`b_tree`](src/dsa/trees/b_tree.rs) | B-Tree | Multi-way search tree, proactive splitting |
| [`splay_tree`](src/dsa/trees/splay_tree.rs) | Splay Tree | Self-adjusting, amortized O(log n) |
| [`treap`](src/dsa/trees/treap.rs) | Treap | Randomized BST + heap, expected O(log n) |
| [`two_three_tree`](src/dsa/trees/two_three_tree.rs) | 2-3 Tree | Perfectly balanced, conceptual basis for RB trees |
| [`patricia_trie`](src/dsa/trees/patricia_trie.rs) | Patricia Trie | Path compression, prefix search, Ethereum MPT analogy |

#### Graphs (`dsa::graphs`)

| Module | Data Structure | Key Feature |
|--------|---------------|-------------|
| [`adjacency_list`](src/dsa/graphs/adjacency_list.rs) | Weighted Directed Graph | BFS/DFS, generic over vertex/weight types |
| [`dag`](src/dsa/graphs/dag.rs) | Directed Acyclic Graph | Cycle prevention, Kahn's + DFS topological sort |

#### Algorithms (`dsa::algorithms`)

| Module | Algorithm | Key Feature |
|--------|-----------|-------------|
| [`binary_search`](src/dsa/algorithms/binary_search.rs) | Binary Search | Iterative, recursive, and BST variants |
| [`bubble_sort`](src/dsa/algorithms/bubble_sort.rs) | Bubble Sort | Early termination, comparison with other sorts |
| [`bfs`](src/dsa/algorithms/bfs.rs) | Breadth-First Search | Level-order traversal, by-level grouping |
| [`dfs`](src/dsa/algorithms/dfs.rs) | Depth-First Search | Pre/in/post-order, iterative + recursive |

---

### `leetcode/` — 21 Curated Problems by Pattern

#### Sliding Window (`leetcode::sliding_window`)

| # | Problem | Difficulty | Key Technique |
|---|---------|-----------|---------------|
| 76 | [Minimum Window Substring](src/leetcode/sliding_window/minimum_window_substring.rs) | Hard | Two-pointer + frequency map |
| 3 | [Longest Substring No Repeat](src/leetcode/sliding_window/longest_substring_no_repeat.rs) | Medium | Last-seen map |
| 239 | [Sliding Window Maximum](src/leetcode/sliding_window/sliding_window_maximum.rs) | Hard | Monotonic deque |
| 424 | [Longest Repeating Char Replace](src/leetcode/sliding_window/longest_repeating_char_replace.rs) | Medium | High-water-mark trick |
| 992 | [Subarrays with K Different](src/leetcode/sliding_window/subarrays_k_different.rs) | Hard | atMost(K) - atMost(K-1) |

#### Hashing (`leetcode::hashing`)

| # | Problem | Difficulty | Key Technique |
|---|---------|-----------|---------------|
| 1 | [Two Sum](src/leetcode/hashing/two_sum.rs) | Easy | Complement lookup |
| 49 | [Group Anagrams](src/leetcode/hashing/group_anagrams.rs) | Medium | Canonical key (sorted chars) |
| 560 | [Subarray Sum Equals K](src/leetcode/hashing/subarray_sum_equals_k.rs) | Medium | Prefix sum + count map |
| 128 | [Longest Consecutive Sequence](src/leetcode/hashing/longest_consecutive_sequence.rs) | Medium | HashSet start detection |
| 438 | [Find All Anagrams](src/leetcode/hashing/find_all_anagrams.rs) | Medium | Rolling frequency array |
| 525 | [Contiguous Array](src/leetcode/hashing/contiguous_array.rs) | Medium | 0→-1 transform + prefix sum |

#### Tree Traversal (`leetcode::tree_traversal`)

| # | Problem | Difficulty | Key Technique |
|---|---------|-----------|---------------|
| 208 | [Implement Trie](src/leetcode/tree_traversal/implement_trie.rs) | Medium | Fixed-size array children |
| 211 | [Add and Search Words](src/leetcode/tree_traversal/add_search_words.rs) | Medium | DFS wildcard matching |
| 212 | [Word Search II](src/leetcode/tree_traversal/word_search_ii.rs) | Hard | Trie + backtracking with pruning |
| 297 | [Serialize/Deserialize BT](src/leetcode/tree_traversal/serialize_deserialize_bt.rs) | Hard | Pre-order with null markers |
| 124 | [Binary Tree Max Path Sum](src/leetcode/tree_traversal/binary_tree_max_path_sum.rs) | Hard | Post-order gain propagation |
| 235 | [LCA of BST](src/leetcode/tree_traversal/lca_bst.rs) | Medium | BST-property traversal |

#### Graph Traversal (`leetcode::graph_traversal`)

| # | Problem | Difficulty | Key Technique |
|---|---------|-----------|---------------|
| 210 | [Course Schedule II](src/leetcode/graph_traversal/course_schedule_ii.rs) | Medium | Kahn's + DFS topo sort |
| 269 | [Alien Dictionary](src/leetcode/graph_traversal/alien_dictionary.rs) | Hard | Implicit edge extraction |
| 200 | [Number of Islands](src/leetcode/graph_traversal/number_of_islands.rs) | Medium | DFS flood fill |
| 126 | [Word Ladder II](src/leetcode/graph_traversal/word_ladder_ii.rs) | Hard | BFS + backtrack |
| 332 | [Reconstruct Itinerary](src/leetcode/graph_traversal/reconstruct_itinerary.rs) | Hard | Hierholzer's Eulerian path |
| 1192 | [Critical Connections](src/leetcode/graph_traversal/critical_connections.rs) | Hard | Tarjan's bridge finding |

#### Recursion & Divide-and-Conquer (`leetcode::recursion_divide_conquer`)

| # | Problem | Difficulty | Key Technique |
|---|---------|-----------|---------------|
| 23 | [Merge K Sorted Lists](src/leetcode/recursion_divide_conquer/merge_k_sorted_lists.rs) | Hard | Pairwise D&C merge |
| 53 | [Maximum Subarray](src/leetcode/recursion_divide_conquer/maximum_subarray.rs) | Medium | Kadane's + D&C |
| 91 | [Decode Ways](src/leetcode/recursion_divide_conquer/decode_ways.rs) | Medium | DP with O(1) space |
| 341 | [Flatten Nested List Iterator](src/leetcode/recursion_divide_conquer/flatten_nested_list_iterator.rs) | Medium | Lazy Iterator with stack |

---

## 🚀 Quick Start

```bash
# Build
cargo build

# Run all tests (196 unit tests + 14 doc-tests)
cargo test

# Lint
cargo clippy
```

## 📐 Design Principles

- **Educational first**: Every file has a header explaining what, why, complexity, and blockchain analogies
- **Self-contained**: No external dependencies — `std` only
- **Idiomatic Rust**: `Box<T>` for ownership, `Option` for nullable, `Ord` for comparison
- **Tested**: Every module has `#[cfg(test)] mod tests` with edge cases
- **Edition 2024**: Uses latest Rust edition features
