# Solidity counter demo

From `pow-node`, run `cargo test -p node solidity_counter_demo`. The test starts
the local miner, deploys `Counter.bytecode`, sends `set(42)`, and checks storage
and the emitted event. The checked-in bytecode was compiled from `Counter.sol`
with Solidity 0.8.26 targeting Shanghai. To rebuild it, run `forge build` in this
directory and copy `bytecode.object` from `out/Counter.sol/Counter.json` to
`Counter.bytecode`.
