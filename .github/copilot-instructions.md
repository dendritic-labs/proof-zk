# ProofZK Project Instructions

This is a DID-based privacy service MVP for airlines and genomics applications.

## Core Principles
- No central data storage - apps request proofs instead of storing data
- Integration with existing wallets (Apple/Google/Samsung)
- Selective disclosure by default (prove age vs store birthdate)
- Ephemeral relay for temporary data access that disappears after transactions

## Project Structure
- `/core` - DID management and zero-knowledge proof system
- `/wallet-integration` - Wallet connectivity layer
- `/relay` - Ephemeral data relay service
- `/demos` - Use case demonstrations (airline, genomics)
- `/docs` - Documentation and API references

## Development Guidelines
- Focus on privacy-first architecture
- Minimize data retention
- Use ZK proofs for verification
- Implement selective disclosure patterns