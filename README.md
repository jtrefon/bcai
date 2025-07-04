# Detailed System Architecture Document
_Version 0.1 – this document provides a technical deep dive into the architecture of the decentralized AI training blockchain, expanding on the whitepaper with more implementation-oriented details and diagrams._
### Mission
Our mission is to deliver the fastest and most secure decentralized platform for AI model training. By uniting hardware around the world, we aim to become the number one resource for training models without centralized bottlenecks. The network prioritizes raw performance and security while enabling virtually unlimited hardware access through decentralization.
**Note:** This repository is an early prototype. Many components remain incomplete or simulated. See
- [HONEST_GAP_ANALYSIS.md](HONEST_GAP_ANALYSIS.md) (governance gap analysis)
- [HONEST_GAP_ANALYSIS_AI.md](HONEST_GAP_ANALYSIS_AI.md) (AI training & data transfer gap analysis)
- [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) (realistic plan & phase gaps)
- [POUW_IMPLEMENTATION_PLAN.md](POUW_IMPLEMENTATION_PLAN.md) (steps to implement PoUW)
for the current status.


## Network Topology and Node Interconnectivity
The network topology consists of a main blockchain network and an overlay network for training communications. All nodes connect to the main P2P network, which propagates blocks and transactions (including training job postings and results). This main net uses standard gossip protocols adapted from popular blockchain clients, ensuring reliability and resistance to partitioning. 

For the heavy data exchange required in training (e.g., sharing model parameters, gradients, data shards), the nodes form an overlay mesh network. When a training job starts, participating worker nodes open direct channels to each other (or via a designated coordinator). This can be implemented using a distributed hash table (DHT) to discover peers or leveraging existing libraries (Libp2p streams, gRPC, etc.) for structured group communication. In practice, we envision using something like libp2p’s pub-sub mechanism to create a topic for each training job, where only involved nodes subscribe and publish updates. This way, model updates aren’t broadcast to the entire network (saving bandwidth), only to those who need it. 

**Node discovery:** New nodes join by connecting to hardcoded bootstrap peers and then using a peer discovery protocol (like Kademlia DHT). They exchange info about their capabilities which gets stored in a lightweight Node Metadata chain or DHT. We might keep an on-chain registry for staking and identity, but the detailed hardware specs can be kept off-chain to avoid bloating the chain (with periodic hashed attestations to prevent forgery). The architecture ensures that at any time, a node looking for, say, “4 GPUs available” can query the network and find a list of node IDs with those specs. 

**High-level diagram:** The main blockchain acts as the backbone. Clients connect to any full node to submit tasks or read results. Full nodes (which include trainers, evaluators, etc.) propagate these. Once a task is accepted by certain trainers, they form a mini-network among themselves (plus evaluators and potentially a supervisor) for the duration of that task. One can imagine it as a hub-and-spoke: the blockchain is the hub where tasks and final results pass, and the spokes are ephemeral task-specific networks doing the actual compute. (If figure illustration was available, here we would include a diagram of nodes connected to the blockchain, and a highlighted subset of nodes engaged in an off-chain training session, exchanging data among themselves.)

## Smart Contract System and Workflow
The smart contract layer (or substrate runtime modules) dictates the logic of job lifecycle and payment. Here’s a breakdown of key contracts/modules:
* **JobManager Contract:** When a client posts a job, it’s recorded by JobManager. It holds the job details, the escrowed payment, and tracks the state (open, in-progress, finished). It has functions like postJob(spec, payment), assignWorkers(jobID, workerList), submitResult(jobID, modelHash, metrics) and distributePayment(jobID). This contract ensures that only authorized evaluators can mark a job as completed and trigger payment release.
* **WorkerRegistry Contract:** This contract tracks registered worker nodes. Workers register themselves (possibly staking a minimum token amount) and list their capabilities (some of these may be stored as events or off-chain to avoid chain bloat). The registry could also store each worker’s reputation score and a record of slashes if any. It might integrate with governance to gate membership (e.g., maybe initially only whitelisted or known participants while bootstrapping, then opened up gradually).
* **Staking/Slashing Contract:** A generic contract that holds stakes for each node and implements slashing logic. For instance, if an evaluator finds a worker cheated, it calls slash(worker, penalty) on this contract, transferring a portion of the worker’s stake to a burn address or to the evaluator as a reward. Similarly, if evaluators misbehave, there’s a mechanism to slash their stake. The rules for slashing are pre-defined and ideally automated: e.g., if consensus on a bad result is reached, it automatically invokes slashing.
* **Governance Contract:** Not directly part of the training workflow, but crucial for upgrades. It might hold the logic for proposing and voting on changes, including parameter adjustments to the above contracts (like changing fee percentages, etc.).

**Custom Training VM:** Our architecture uses a dedicated virtual machine optimized for AI tasks instead of EVM or Substrate-based runtimes. This removes unnecessary overhead and lets us embed GPU-focused instructions directly in the runtime. All job logic, staking and payments run in this VM. **Fee Mechanism:** Every job posting involves an on-chain transaction with a small fee to discourage spam and reward validators.

## Consensus and Block Details
**Block structure:** Blocks in this blockchain contain, aside from regular transactions, training-related metadata. Building on ideas from the PoUW paper (Project PAI’s design), a block header might be extended to include:
* a reference to the current training task being used for PoUW (task ID),
* a useful work proof field (could be a hash that proves a certain number of training iterations were done),
* perhaps the final model hash if the block concludes a training task.

For example, whereas Bitcoin block header has a nonce that miners adjust, here miners might adjust some component of the model training (or a dummy nonce plus model parameters) to satisfy a difficulty target. One proposed scheme: define the block’s hash as H(previousHash, txRoot, modelHash, nonce), and require that this hash is below a target. The trick is to link modelHash with actual useful work – e.g., modelHash could be the hash of the neural network weights after N training iterations. Miners can’t freely choose that; they have to actually perform training to get a good model, which in turn influences modelHash. We ensure that without doing the training, they have no shortcut to produce a valid hash under target (this is a complex area and likely will evolve with research). 

**Hybrid PoS integration:** If we use PoS as well, the block proposer might be chosen by stake, then required to include a useful work proof as an extra step (so both conditions must be satisfied). Or we alternate: some blocks come from PoUW, some from PoS, or do a weighted random mix (for example, every 10th block must be PoUW, others PoS). These details need tuning to balance security and performance. 
**Consensus Protocol:** Initially, we may adopt a classical BFT-style consensus (like Tendermint or Aura/Grandpa in Substrate) if going PoS, to get immediate finality. If pure PoW/PoUW, we inherit Nakamoto-style longest chain with probabilistic finality. A hybrid might use checkpoints where PoS validators finalize epochs of PoUW-mined blocks. 
**Subnets for Specialized Tasks:** A concept borrowed from Bittensor is having specialized sub-networks for different AI tasks. In our architecture, this could manifest as shards or parachains dedicated to particular model types (e.g., one subnet specialized for NLP model training, another for computer vision, etc.). Each could have customized parameters (e.g., the NLP subnet might allow larger model sizes and have longer block times to accommodate that training). These subnets would all be tied to the main chain via a relay or shared security model (Polkadot-style, perhaps). This adds complexity, so in version 1 we likely focus on a single chain, but we keep the door open for scaling horizontally via subnets if the demand grows (so the architecture’s modular design will allow plugging in new sub-chains that interoperate with the main token).

## Data Management and Model Handling
**Data storage:** Datasets can be huge (gigabytes). To avoid third-party bottlenecks we are building an integrated distributed storage layer for the network. Training data and model artifacts are stored there and referenced on-chain by content hashes.
* Another idea: if data is sensitive, client could secret-share or encrypt it such that only chosen workers get the key. But that’s at the application layer.

The model weights themselves, after training, might be tens or hundreds of megabytes (for big neural nets). We also won’t store those fully on-chain. Instead, the final model is stored in the network's distributed storage and its hash recorded on-chain as the official result artifact. The blockchain thus contains pointers to models rather than models themselves. This is analogous to how NFT metadata is stored off-chain with on-chain hash references. 

**Model format and verification:** We standardize that all models produce a final hash (like SHA-256) of their weights and perhaps a hash of evaluation metrics. Evaluators sign the hash of the model they validated. So anyone later can retrieve the model by that hash (from the storage layer), and check the signature and know it's the one that was verified. Optionally, if someone wants to re-train or fine-tune that model, they can use it as a starting point – in future, chaining tasks might be possible (the result of one job becomes the starting model of another job). 
**Data availability:** Blockchains worry about data availability for validation. In our case, the critical data for consensus is relatively small (model hashes, proofs). The large training data is not needed by everyone, only the workers. We must ensure workers actually had the data. If a worker claims “I trained the model”, but maybe they only had half the data? That’s fine—they’d just get worse accuracy. The evaluators catch if the model is bad. If the data itself was unavailable or incomplete, the job fails and maybe is retried or cancelled. So the system is somewhat robust to data unavailability – it just results in no progress and the client might need to repost (with maybe better data distribution next time). 

**Parallel and Federated Training:** The architecture can support different training paradigms:
* **Synchronous parallel:** All trainers get different slices of data, they periodically sync gradients (like data-parallel training). This requires a bit of orchestration (like a parameter server or all-reduce among them). We might designate the block proposer or a supervisor node as the interim “parameter server” to collect and redistribute gradients each round. This adds complexity but is doable via the overlay network.
* **Asynchronous or federated:** Each trainer trains on their data independently and occasionally shares model updates that are averaged. This is easier network-wise (no strict barrier synchronization), but perhaps slower to converge. The blockchain might simply require each trainer to submit a model update transaction periodically, and a smart contract aggregates them (computing an average). This could be heavy on-chain if done frequently, so more likely it’s done off-chain and only final results are on-chain.
* **Competitive training:** Where multiple trainers try different hyperparameters or even different model architectures (like an AutoML search), and evaluators pick the best. The architecture handles this by treating each variant as part of the same task group; evaluators score each, and one wins the reward (others might get partial compensation for trying, if the client chooses to encourage diversity).

The architecture documentation would typically include sequence diagrams showing these interactions for clarity: e.g., a timeline of a training job from posting to completion, indicating on-chain events vs off-chain communications. 

_(If available, an embedded sequence or flow diagram would be presented here to illustrate the step-by-step interactions between client, smart contracts, trainer nodes, evaluator nodes, and the blockchain at various phases.)_

## Example Use Case Walkthrough
To make it concrete, consider an example: A researcher wants to train a convolutional neural network on a medical images dataset using our network.
1. **Job Posting:** The researcher (client) formats the model (perhaps as an ONNX file or code reference), encrypts their dataset and uploads to the storage layer, then calls JobManager.postJob with parameters: model type=CNN, dataHash=<storage hash>, dataKey=<encrypted key for evaluators>, required GPU memory=16GB, reward=1000 TRAIN tokens. This transaction goes into block X.
2. **Matching:** Within a few blocks, three worker nodes see the posted job (they constantly monitor new jobs from JobManager events). These three each have >=16GB GPUs and are interested (the reward is good). They call JobManager.assignWorker(jobID) to volunteer, which requires them to lock some stake as well. The JobManager now marks the job as having 3 workers assigned and closes assignment (assuming it needed 3 as per spec).
3. **Training Phase:** Off-chain, these 3 workers form a mesh. They retrieve the data from the storage layer (since they have the hash) – since it’s encrypted, they can start training on it if the model training doesn’t require plaintext (if it’s normal training, they might need the key; perhaps the data isn't fully secret in this example, or the client trusts enclaves). They start training, syncing their model every few minutes. They also produce intermediate checkpoint hashes. They perhaps agree one of them will be the block proposer for blocks during this training (or whichever finds a PoUW nonce solution first proposes the block).
4. **Block Production:** Suppose after some iterations, one worker finds a valid block proof (meeting difficulty). It includes the current model hash in the block and gets it accepted. That block also contains transactions of intermediate results. Training continues over multiple blocks maybe.
5. **Completion:** After say 10 epochs, they reach the target or the budget runs out. The workers stop and one final model is produced (maybe they all have identical final model if synchronous training). They each submit a transaction submitResult(jobID, modelHash) along with a commitment to accuracy (they might guess or compute it roughly). Now the evaluators (maybe two evaluators were auto-assigned at job start by the protocol from a pool of available validators) get the encrypted test data key from the JobManager (which stored it from client). Evaluators fetch the final model from one of the workers (or the storage layer if it was uploaded), run it on the test set, and get accuracy, say 85%.
6. **Verification and Payment:** Evaluators submit their signed attestation completeJob(jobID, accuracy=85%, modelHash=XYZ, verdict=success) on-chain. The JobManager sees two out of two evaluators agree within a threshold, so it marks job as Success and triggers distributePayment(jobID). The 1000 TRAIN is unlocked: 900 go to the 3 workers (split based on, say, their proportion of data processed or equal split if equal work), 50 go to the evaluators (25 each) as a service fee, 50 go to the network treasury or burnt as fee (for instance). The stake of workers is returned (they weren’t slashed since all good).
7. The researcher can now fetch the model from the storage layer (hash XYZ) and use the key to decrypt if needed (in this scenario maybe data wasn’t super secret or training proceeded). Everyone is happy: the researcher got their model for 1000 tokens, the workers earned tokens for their computational work, and the network advanced by one more useful block of work.

This example demonstrates the interplay of on-chain and off-chain flows in our architecture.

## Scalability and Future Enhancements
The architecture as described should handle moderate load (especially by offloading heavy compute off-chain), but to truly scale to AI cloud levels, we have plans:
* Implement sharding or sidechains for different task categories, as mentioned.
* Utilize Layer-2 networks or rollups for micro-transactions – e.g., micropayments between workers could be done off-chain, settling net results on-chain to reduce bloat.
* Possibly integrate zero-knowledge proofs for verifying computation (ZK-SNARKs that prove “I performed X training iterations correctly”). This is cutting-edge and heavy for large models, but the field is progressing. If feasible, it would reduce reliance on multiple evaluators – a single proof could convince all verifiers.
* **Edge integration:** allow IoT or smartphones to contribute smaller tasks or use the trained models, via light client protocols.
* **AI-specific instructions:** as we control the VM, we can add more optimized instructions (like a matrix multiplication opcode that directly leverages GPU) beyond what Cortex did. Also possibly integrate with emerging AI accelerators or even quantum hardware in future (modular hardware abstraction).

This detailed architecture will continue to evolve with implementation and testing. The above serves as a blueprint for engineers to begin building components and for auditors to understand the intended system interactions. It is designed with modularity in mind – each part (networking, consensus, contracts, off-chain worker) can be developed and improved somewhat independently, as long as the interfaces (like the transactions and message formats) are respected. (End of Architecture Document. In a real publication, one might include appendices with API definitions or pseudocode of critical algorithms like the training consensus or the reward distribution logic.)

## Code of Conduct
Please read our [Code of Conduct](CODE_OF_CONDUCT.md) for guidelines on expected behavior and the moderation process.

For a breakdown of near-term development tasks, see [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).
We are also hosting a [community challenge](COMMUNITY_CHALLENGE.md) to test the network and reward participants with testnet tokens.

### Prototype CLI
This repository now includes a small Rust program under `jobmanager/` that
demonstrates how jobs could be posted and managed locally. It is an early
prototype meant to accompany the roadmap items and can be built with
`cargo build` inside that directory.

For generating account keys there is also a simple tool in `keygen/`:

```bash
cargo run --manifest-path keygen/Cargo.toml --
```
This prints a new Ed25519 keypair in JSON that can be saved for your first
transactions.

### VM Runtime
An experimental VM implementation lives under `runtime/`. It currently supports
basic stack-based arithmetic instructions and serves as the foundation for a
future AI-optimized runtime.

Build and run its tests with:

```bash
cargo test --manifest-path runtime/Cargo.toml
```

### Coding Standards
We maintain coding guidelines in [CODING_STANDARDS.md](CODING_STANDARDS.md)
to encourage SOLID, clean architecture and consistent formatting across
all crates. Please review these before contributing.
