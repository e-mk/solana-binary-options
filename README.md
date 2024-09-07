## About
This Solana program showcases the tools and techniques used to create Binary Options on the Solana blockchain.

The program is structured around three key instructions:

- **initialize**: The instruction for the market maker to initialize the Binary Option.
- **do_bet**: The instruction to place a bet.
- **resolve**: The final step, where the program distributes prizes and closes the accounts.

## How to run
1. Install solana development suit. 
2. Install the dependencies:
   
    ```sh
    anchor build
    ```   
3. Build:
    ```sh
    anchor test
    ```

## Main Challenges
- Scheduling instruction execution on the Solana chain proved to be challenging. The main approach involved using the `clockwork-sdk`, which is now deprecated and incompatible with the latest versions of Anchor. Considering this, the most efficient solution is to schedule the `resolve` instruction execution off-chain.
- Pyth Oracle does not support Solana Devnet, leading to mocked prices for tests. Other Oracles were not explored due to time constraints.
