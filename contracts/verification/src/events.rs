#![allow(deprecated)]
use soroban_sdk::{Address, Env, String, Symbol};

pub const MILESTONE_APPROVED: &str = "milestone_approved";
pub const VALIDATOR_REGISTERED: &str = "validator_registered";
pub const VALIDATOR_REVOKED: &str = "validator_revoked";
pub const CONTRACT_PAUSED: &str = "contract_paused";
pub const CONTRACT_UNPAUSED: &str = "contract_unpaused";
pub const CONTRACT_INITIALIZED: &str = "contract_initialized";
pub const PROGRESS_CONTRACT_UPDATED: &str = "progress_contract_updated";
pub const DISPUTE_RESOLVED: &str = "dispute_resolved";
pub const ADMIN_TRANSFER_PROPOSED: &str = "admin_transfer_proposed";
pub const ADMIN_TRANSFERRED: &str = "admin_transferred";

pub fn admin_transfer_proposed(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (Symbol::new(env, ADMIN_TRANSFER_PROPOSED),),
        (old_admin.clone(), new_admin.clone()),
    );
}

pub fn admin_transferred(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (Symbol::new(env, ADMIN_TRANSFERRED),),
        (old_admin.clone(), new_admin.clone()),
    );
}

pub fn milestone_approved(
    env: &Env,
    player_id: u64,
    validator: &Address,
    milestone_index: u32,
    description: &String,
    evidence_hash: &String,
) {
    env.events().publish(
        (
            Symbol::new(env, "milestone_approved"),
            validator.clone(),
            milestone_index,
        ),
        (player_id, description.clone(), evidence_hash.clone()),
    );
}

pub fn validator_registered(env: &Env, wallet: &Address, credentials: &String) {
    env.events().publish(
        (Symbol::new(env, "validator_registered"), wallet.clone()),
        (wallet.clone(), credentials.clone()),
    );
}

pub fn validator_revoked(env: &Env, wallet: &Address, reason: &String) {
    env.events().publish(
        (Symbol::new(env, "validator_revoked"),),
        (wallet.clone(), reason.clone()),
    );
}

pub fn validator_restored(env: &Env, wallet: &Address) {
    env.events()
        .publish((Symbol::new(env, "validator_restored"),), wallet.clone());
}

pub fn validator_transferred(env: &Env, old_wallet: &Address, new_wallet: &Address) {
    env.events().publish(
        (Symbol::new(env, "validator_transferred"),),
        (old_wallet.clone(), new_wallet.clone()),
    );
}

pub fn contract_paused(env: &Env, admin: &Address) {
    env.events()
        .publish((Symbol::new(env, "contract_paused"),), admin.clone());
}

pub fn contract_unpaused(env: &Env, admin: &Address) {
    env.events()
        .publish((Symbol::new(env, "contract_unpaused"),), admin.clone());
}

pub fn contract_initialized(env: &Env, admin: &Address) {
    env.events()
        .publish((Symbol::new(env, "contract_initialized"),), admin.clone());
}

pub fn progress_contract_updated(env: &Env, progress_contract: &Address) {
    env.events().publish(
        (Symbol::new(env, "progress_contract_updated"),),
        progress_contract.clone(),
    );
}

/// Emitted when a player disputes a milestone (issue #471)
pub fn milestone_disputed(env: &Env, player_id: u64, milestone_index: u32, reason: &String) {
    env.events().publish(
        (
            Symbol::new(env, "milestone_disputed"),
            player_id,
            milestone_index,
        ),
        reason.clone(),
    );
}

/// Emitted when an admin resolves a milestone dispute.
pub fn dispute_resolved(env: &Env, player_id: u64, milestone_index: u32, upheld: bool) {
    env.events().publish(
        (
            Symbol::new(env, "dispute_resolved"),
            player_id,
            milestone_index,
        ),
        upheld,
    );
}

/// Emitted when a milestone is recorded but level advancement is skipped because
/// the player is already at the maximum level (EliteTier).  The milestone itself
/// is still persisted; only the cross-contract advance_level call is omitted.
/// `reason` is always "AlreadyAtMaxLevel".
pub fn level_advancement_skipped(env: &Env, player_id: u64, reason: &String) {
    env.events().publish(
        (Symbol::new(env, "level_advancement_skipped"), player_id),
        reason.clone(),
    );
}

/// Emitted when level advancement is skipped because the progress contract
/// address has not been configured.  Common during testing without a full
/// deployment.  In production this indicates a missing wiring step and the
/// indexer should alert on it.  The milestone is still persisted.
pub fn progress_contract_not_set(env: &Env, player_id: u64) {
    env.events().publish(
        (Symbol::new(env, "progress_contract_not_set"), player_id),
        (),
    );
}

/// Emitted just before a ProgressCallFailed error is returned, so the
/// off-chain indexer can detect the failure by scanning transaction receipts.
/// Because ProgressCallFailed aborts the entire transaction, this event only
/// appears in the diagnostic stream — it is not committed to the ledger.
/// Payload is the raw error discriminant returned by try_advance_level.
pub fn progress_call_failed(env: &Env, player_id: u64, error_code: u32) {
    env.events().publish(
        (Symbol::new(env, "progress_call_failed"), player_id),
        error_code,
    );
}
