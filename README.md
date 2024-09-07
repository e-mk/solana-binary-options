## About
This Solana program is designed to show some tools and techniques that are used to create Binary Options on the Solana blockchain. 

The program is organized into three instructions:

- **initialize**: The instruction for the market maker to init the Binary Option
- **do_bet**: The instruction to make the bet
- **resolve**: This is the final step where the program distributes the prize and closes the accounts.

## Main Challenges
- Scheduling the instruction execution on the Solana chain was challenging. The main approach is to use `clockwork-sdk` which is deprecated and does not support Anchor's latest versions. Taking that into account the most officiant way will be to schedule `resolve` instruction execution off-chain.
- Pyth Oracle is not supporting Solana devnet, resulting in mocking price on devnet for tests. Other Oracles were not considered because of the time concerns.

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
