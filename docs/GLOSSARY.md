# ScoutChain Glossary

Definitions for key domain terms, on-chain types, and contract concepts used
across the ScoutChain codebase, documentation, and API reference. Entries are
listed alphabetically.

---

## Admin

The privileged Stellar account that owns a contract. Set once during
`initialize` and transferable (irreversibly) via `transfer_admin`. Only the
admin may call fee management, validator registry, and circuit-breaker
functions.

---

## advance_level

The `progress` contract function that increments a player's `ProgressLevel` by
exactly one tier (0→1, 1→2, or 2→3). Called atomically by
`verification.approve_milestone` (for levels 1 and 2) and, for level 3, either
directly by `scout_access.log_trial_offer` or via a `confirm_trial_offer` flow.
Cannot skip tiers; attempting to advance beyond `EliteTier` returns
`AlreadyAtMaxLevel`.

See also: [ProgressLevel](#progresslevel), [Trial Offer](#trial-offer).

---

## approve_milestone

The `verification` contract function called by a registered validator to confirm
a player achievement. Records a `Milestone` on-chain and immediately
cross-calls `progress.advance_level`, making the milestone approval and the
level change atomic in the same Stellar transaction.

See also: [Validator](#validator), [Milestone](#milestone).

---

## batch_contact_players

A `scout_access` function that allows a scout to unlock multiple player
profiles in a single transaction. Contact fees are charged once per new player;
already-contacted players are silently skipped. Pro-tier monthly contact
quotas are enforced across the batch.

---

## Circuit Breaker

The `pause_contract` / `unpause_contract` admin mechanism present on all four
contracts. While paused, all state-changing operations revert with
`ContractPaused`; read-only queries remain available. Designed for emergency
response without state loss.

---

## CID (Content Identifier)

An IPFS or Arweave content address used to reference off-chain documents
(highlight reels, evidence, trial offer terms). Two formats are accepted:

- **CIDv0** — starts with `Qm`, exactly 46 characters, base58btc encoding.
- **CIDv1** — starts with `bafy`, 59–128 characters, base32 encoding.

Validation is enforced on-chain in `register_player`, `approve_milestone`, and
`log_trial_offer` via `validate_cid`.

---

## ContactRecord

A persistent storage entry in `scout_access` keyed by `(player_id, scout)`
that records that a scout has paid to contact a player. Prevents duplicate
charges on repeated `pay_to_contact` calls.

---

## EliteTier

`ProgressLevel` 3 — the highest tier. A player reaches `EliteTier` when an
Elite-tier scout logs a trial offer on-chain via `log_trial_offer`. No further
`advance_level` calls are possible; `AlreadyAtMaxLevel` is returned for any
subsequent attempt.

---

## FeeConfig

Platform fee configuration stored in `scout_access` instance storage. Contains
stroops values for each subscription tier, the per-contact fee, the
subscription duration in seconds, and the Pro-tier monthly contact limit. All
numeric fields must be positive; updated by admin via `update_fee_config`.

---

## Milestone

A verified player achievement recorded by the `verification` contract. Each
`Milestone` stores the player ID, the approving validator's address, a
description, an IPFS/Arweave evidence CID, the approval timestamp, and the
ledger sequence number for tamper-proof auditability.

---

## pay_to_contact

A `scout_access` function that charges `contact_fee_stroops` XLM to unlock a
player's contact details. Requires an active, non-expired subscription. Pro
scouts are limited to `pro_contact_limit` contacts per calendar month.

---

## ProgressEntry

An immutable history record appended to the `progress` contract every time a
player's level changes (advance or reset). Each entry stores the old and new
`ProgressLevel`, the authorizing caller, the Unix timestamp, the `milestone_ref`
index, and the ledger sequence number.

---

## ProgressLevel

The four-tier player verification scale:

| Integer | Variant               | Meaning                                                |
|---------|-----------------------|--------------------------------------------------------|
| 0       | `Unverified`          | Profile created; no verifications yet                  |
| 1       | `VerifiedIdentity`    | Identity confirmed by a validator                      |
| 2       | `PerformanceMilestones` | Performance stats verified by an approved validator  |
| 3       | `EliteTier`           | Trial offer logged by an Elite scout                   |

Valid transitions: **0 → 1 → 2 → 3** (sequential only). Skipping or reversing
levels is rejected with `InvalidProgressTransition` except when an admin calls
`reset_player_level`.

---

## Scout

A registered talent scout who accesses the player pool through the
`scout_access` contract. Scouts subscribe to a tier (`Basic`, `Pro`, or
`Elite`) to unlock contact and trial-offer capabilities.

---

## Subscription

An active `scout_access` record keyed by the scout's wallet address. Contains
the tier, the `expires_at` Unix timestamp, and the `subscribed_at` timestamp.
An expired subscription is treated the same as no subscription — renew via
`subscribe` before performing any contact or trial-offer operations.

---

## SubscriptionTier

Three tiers of scout access:

| Tier    | Accessible Levels | pay_to_contact | log_trial_offer | Monthly Contact Limit |
|---------|-------------------|----------------|-----------------|-----------------------|
| Basic   | Level 1–3         | ❌              | ❌               | N/A                   |
| Pro     | Level 0–3         | ✅              | ❌               | `pro_contact_limit`   |
| Elite   | Level 0–3         | ✅              | ✅               | Unlimited             |

Downgrades while a subscription is still active are rejected
(`SubscriptionDowngradeNotAllowed`). Upgrades are permitted but must observe a
minimum 1-hour interval between `subscribe` calls (`UpgradeTooSoon`).

---

## Trial Offer

An on-chain record — stored as a `TrialOffer` struct — that a registered Elite
scout has extended a formal trial to a player. Logging a trial offer is the
trigger that advances a player to `EliteTier` (Level 3).

### TrialOffer (struct)

```rust
pub struct TrialOffer {
    pub player_id: u64,
    pub scout: Address,
    pub details_hash: String, // IPFS/Arweave CID of the offer document
    pub logged_at: u64,       // Unix timestamp
}
```

`TrialOffer` records are written atomically alongside the cross-contract
`advance_level` call inside `log_trial_offer`. The offer is stored
permanently; the player level advances in the same Stellar transaction.

A per-`(scout, player)` cooldown of 24 hours is enforced: submitting a second
trial offer from the same scout to the same player within 24 hours returns
`TrialOfferRateLimited`.

### TrialEscrow (escrow-and-confirmation flow)

> **Note:** The `TrialEscrow` concept describes the *intended* two-step
> escrow-and-confirmation flow that supersedes the original single-step
> `log_trial_offer` design. It is documented here so that readers relying
> solely on the Glossary understand the full lifecycle, including the escrow
> and refund paths.

In the escrow-and-confirmation model the trial offer lifecycle has two stages:

1. **Escrow stage** (`log_trial_offer`) — The Elite scout calls
   `log_trial_offer`, which creates a `TrialOffer` record and places the offer
   in a *pending* state. At this stage the player's level does **not** advance
   immediately. Instead, the offer awaits explicit confirmation.

2. **Confirmation stage** (`confirm_trial_offer`) — A second call confirms the
   offer (by the player, admin, or another authorised party depending on the
   deployment). Only after confirmation does the contract call
   `progress.advance_level`, advancing the player to `EliteTier`.

The distinction matters for off-chain consumers:

| Property | `TrialOffer` (immediate) | `TrialEscrow` (two-step) |
|----------|--------------------------|--------------------------|
| When level advances | At `log_trial_offer` call | At `confirm_trial_offer` call |
| Player must act | No | Yes (or admin) |
| Offer can expire | No built-in expiry | Yes — `expires_at` deadline enforced |
| Unconfirmed offer refundable | N/A | Yes — scout may reclaim XLM if expired |
| Indexer must handle | Single event | Two events: `trial_offer_logged` + `trial_offer_confirmed` |

**Key distinction for indexers and UI**: if your deployment uses the
escrow-and-confirmation flow, do **not** treat a `trial_offer_logged` event as
proof that the player has reached `EliteTier`. Only a `trial_offer_confirmed`
event guarantees the level has advanced. Always verify the player's current
level via `progress.get_level` before displaying it as `EliteTier`.

See also: [advance_level](#advance_level), [EliteTier](#elitetier),
[ProgressLevel](#progresslevel).

---

## Validator

A trusted on-chain actor — typically a local coach, academy director, or
certified trainer — registered by the admin in the `verification` contract.
Only active validators may call `approve_milestone`. Validators can be revoked
(`revoke_validator`) but not deleted; revoked validators remain in the registry
with `active: false`.

---

## VerifiedIdentity

`ProgressLevel` 1. Reached when a validator calls `approve_milestone` confirming
a player's identity (e.g. active club membership or KYC). Scouts with a Basic
subscription or higher can view players at this level and above.

---

## PerformanceMilestones

`ProgressLevel` 2. Reached after a second `approve_milestone` call confirming
performance statistics or match footage verified by an approved validator.
Pro and Elite scouts can view and contact players at this level.

---

## Unverified

`ProgressLevel` 0. The starting state for every newly registered player.
The player has created a profile and uploaded data but no validator has yet
confirmed their identity. Only Pro and Elite scouts can view players at Level 0.

---

## validate_cid

A shared utility in `scoutchain-shared-types` that checks whether a `String`
is a valid IPFS CIDv0 (`Qm…`, 46 chars) or CIDv1 (`bafy…`, 59–128 chars).
Used by `approve_milestone` and `log_trial_offer` to reject malformed
content-address arguments before writing to persistent storage.

---

## XLM / Stroops

The native Stellar asset used for all on-chain fee payments in ScoutChain.
Contract amounts are always expressed in **stroops** (1 XLM = 10,000,000
stroops) to avoid floating-point handling in WASM contracts. All `FeeConfig`
fields ending in `_stroops` are denominated in stroops.
