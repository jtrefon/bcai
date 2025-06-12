# RFC Document and Community Memorandum
**Title:** RFC-001: Decentralized AI Training Network – Open Design Proposal
**Status:** Draft – for community review
**Target Audience:** Developers, Researchers, and Contributors interested in blockchain or AI
**Discussion:** This RFC is open on our GitHub repository [jtrefon/bcai](https://github.com/jtrefon/bcai) for comments, pull requests, and contributions.

## Introduction
We are excited to present the first Request for Comments (RFC) for our Decentralized AI Model Training Blockchain. This document serves as both a technical specification outline and a call to action for the community. We invite feedback on all aspects of the design – from consensus mechanism details to API specifications – as we refine the protocol together. The goal of this RFC is to ensure transparency in our thought process and to crowdsource expertise from the wider blockchain and AI communities.

## Motivation
Traditional blockchains have successfully decentralized finance; our aim is to decentralize something equally powerful: artificial intelligence model training. By doing so, we unlock innovation and reduce barriers in AI development. We recognize that achieving this is a complex, multi-disciplinary effort. Therefore, we are adopting an open RFC process (inspired by Ethereum’s EIPs and other open standards) to gather the best ideas and ensure our design stands up to scrutiny.

## Proposal Summary
RFC-001 outlines the high-level architecture (as described in the whitepaper section above) including:
* A useful work consensus mechanism that turns model training into the equivalent of block mining.
* Design of a custom VM to support heavy computation and GPU integration.
* Networking protocols for job discovery and distribution.
* Smart contract interfaces for posting jobs and rewarding work.
* A token economic model to incentivize participation.
* Security approaches (validation via test datasets, staking, slashing conditions).
* Governance structures for upgrades.
The RFC is not a polished final specification; rather, it’s a starting point meant to evolve with community input. We expect to iterate on this RFC based on comments and eventually either finalize it or break it into multiple focused RFCs (for example, separate RFCs for the consensus, for the VM, etc.).
## How to Contribute
We encourage contributions in the following ways:
* **GitHub Discussions and Issues:** Our repository [jtrefon/bcai](https://github.com/jtrefon/bcai) has an “RFC-001” discussion where anyone can ask questions or propose changes. If you have a specific improvement, you can open an issue or even a pull request modifying the RFC text. All changes will be reviewed by maintainers and the community.
* **Design Meetings:** We will host public virtual meetings (AMA style or more structured) to discuss major points of feedback. Summaries of these meetings will be posted for transparency.
* **Prototype & Code Contributions:** Alongside the RFC text, we welcome experimental code. For instance, if you have an idea for the consensus algorithm or scheduling, feel free to prototype it and share a link. Early-stage code (even if rough) can validate concepts and will be highly appreciated.
* **Research and References:** If you know of prior research, projects, or data that could inform our design (e.g., a paper on a new proof-of-learning approach, or performance data on GPU scheduling), please share it. Comment in the RFC with references or make a suggestion to include it in our considerations. We aim for an academically and technically rigorous design, so citing known results (as we’ve started doing with PoUW references, etc.) is very welcome.

This RFC will remain in Draft status for an initial review period (proposed: 4-6 weeks), after which we will incorporate the feedback and publish an RFC-001 v2 or mark it as Accepted if consensus is reached. Even after acceptance, further refinement can happen in subsequent RFCs.

## Recruitment and Community
This project thrives on community involvement. We are not just building a network, but a community of decentralized AI enthusiasts. Whether you’re a blockchain dev, a machine learning researcher, or simply a motivated learner, there’s a place for you here. We particularly encourage AI researchers who have felt constrained by compute access – help us design the system you would want to use! 

To stay updated and connect with fellow contributors:

* Follow our lead coordinator J. Trefon on Twitter (@jtrefon) for updates, insights, and coordination of efforts. We regularly post about progress and upcoming discussions there.
* Join our Discord/Telegram (links on GitHub) to chat in real-time. There are channels for each major aspect (consensus, VM, AI/ML, economics, etc.) where domain experts are helping out.
* Check out the “Good First Issues” on our GitHub for tasks that newcomers can tackle to get familiar with the codebase. We mentor new contributors and appreciate every PR, no matter how small.

**Memorandum:** We are actively recruiting contributors and core team members. If you’re passionate about this vision, now is the time to get involved. Early contributors will gain reputation in the project and could potentially join the core development team or foundation driving this initiative. We are looking for Rust/Go developers (for blockchain node coding) machine learning specialists (for integration and validation logic), and community managers. 

Our mission is bold, but with a talented and passionate community, it’s achievable. This RFC marks the first step in designing with the community. Together, we can create a platform that changes how AI models are trained and deployed – making it more open, fair, and innovative. 

We look forward to your feedback and contributions. Let’s build the future of decentralized AI, one RFC at a time! 

(The full text of RFC-001 is available on GitHub, including more detailed technical specifications. By publishing this on GitHub under an open license, we ensure everyone can contribute. This memorandum will also be shared on social platforms and our blog to reach a wider audience.)
