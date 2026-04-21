# Security Policy

## Reporting A Vulnerability

Please do not open public issues that include secrets, private keys, seed
phrases, API keys, vault files, wallet exports, logs, screenshots with secrets,
or full provider response bodies.

Until this project has a public security contact, report issues privately to the
repository owner. Include:

- A short description of the vulnerability.
- Reproduction steps using test data only.
- The expected and actual behavior.
- Any affected version, commit, platform, or terminal environment.

## Sensitive Data Boundaries

`splitbot` stores the SplitNOW API key and wallet secrets in a local encrypted
vault. It can also write plaintext wallet export files when the user chooses an
export command. Treat generated/imported wallets as hot wallets.

The app also communicates with external services, including SplitNOW, Solana
RPC, and CoinGecko. Avoid sharing addresses, order identifiers, or logs unless
you have reviewed them for sensitive data.
