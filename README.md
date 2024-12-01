# AggLayer PessimisticProof

<div align="center">
  <p align="center">
    <a href="http://makeapullrequest.com">
      <img alt="pull requests welcome badge" src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat">
    </a>
    <a href="https://twitter.com/BrianSeong99">
      <img alt="Twitter" src="https://img.shields.io/twitter/url/https/twitter.com/BrianSeong99.svg?style=social&label=Follow%20%40BrianSeong99">
    </a>
  </p>
</div>

This repo explains the design and the usage of Pessimistic Proof in AggLayer. It compares the bench performance of running pessimistic proof in SP1, Valida, and Risc0 zkVMs.

# Architecture of Pessimistic Proof

## 0.Background

### Chains connected on AggLayer

### Security of AggLayer

## 1.Pessimistic Proof Overview

## 2.Data Structure in Pessimistic Proof
Pessimistic Proof exist to compute the state transaction in between bridging events. There are 3 Sparse Merkle Trees that it consumes which are Local Exit Tree, Local Balance Tree, Nullifier Tree, L1 Info Tree. The State Transitions are represented using `bridge_exits`(the bridging transactions) and `imported_bridge_exits`(the claiming transactions).

Let's go through them one-by-one.

### Unified Bridge Data Structure

There's a DETAILED explanation of Unified Bridge data structure writtened in [here](https://github.com/BrianSeong99/AggLayer_UnifiedBridge?tab=readme-ov-file#2-data-structure-in-unified-bridge). You should read the entire section to understand Local Exit Tree, Mainnet Exit Tree, Rollup Exit Tree, Global Exit Root, and L1 Info Tree. 

Here's the TLDR:
- **Local Exit Tree**: All cross-chain transactions using the Unified Bridge are recorded in a Sparse Merkle Tree called Local Exit Tree. Each AggLayer connected chain maintains its own local exit tree.
- **Mainnet Exit Tree**: Mainnet Exit Tree is the same thing as Local Exit Tree, but it is maintained on L1, which tracks the Bridging activities of L1 to all AggLayer connected L2s.
- **Rollup Exit Tree**: Rollup Exit Tree is the Sparse Merkle Tree, where all L2s' Local Exit Root are the leaves of the tree.
- **Global Exit Root**: Global Exit Root is basically the hash of Rollup Exit Root and Mainnet Exit Root. Whenever there's new RER or MER created, it will update the new GER then append it to the L1 Info Tree.
- **L1 Info Tree**: L1 Info Tree is a Sparse Merkle Tree that stores the Global Exit Root.
- **Global Index**: Unique reference of one leaf within a Global Exit Root.

![UnifiedBridge Merkle Tree](./pics/UnifiedBridgeTree.png)

### Local Balance Tree



### Nullifier Tree

### Balance Changes of the chain, Bridge Exits and Imported Bridge Exits



## 3.Pessimistic Proof Components

## 4.Proof Generation & Verification Interface

## 5.Pessimistic Proof Pipeline

# Benchmark on zkVMs

## 1.Benchmark on SP1

## 2.Benchmark on Valida

## 3.Benchmark on Risc0

# References
