#[cfg(not(feature = "imported"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::consts::MsgReplyID;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, IBCLifecycleComplete, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, CONFIG, RECOVERY_STATES};
use crate::{execute, ibc_lifecycle};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:crosschain-swaps";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.debug("crosschain swaps instantiate invoked");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // validate swaprouter contract and owner addresses and save to config
    let swap_contract = deps.api.addr_validate(&msg.swap_contract)?;
    let governor = deps.api.addr_validate(&msg.governor)?;
    let registry_contract = deps.api.addr_validate(&msg.registry_contract)?;
    let state = Config {
        swap_contract,
        governor,
        registry_contract,
    };
    CONFIG.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    deps.api.debug("crosschain swaps migrate invoked");
    match msg {}
}

/// Handling contract execution
#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    deps.api.debug("crosschain swaps execute invoked");
    // IBC transfers support only one token at a time
    match msg {
        ExecuteMsg::OsmosisSwap {
            forward,
            output_denom,
            receiver,
            slippage,
            next_memo,
            on_failed_delivery,
            route,
        } => execute::unwrap_or_swap_and_forward(
            (deps, env, info),
            output_denom,
            slippage,
            &receiver,
            next_memo,
            on_failed_delivery,
            route,
            forward,
        ),
        ExecuteMsg::Recover {} => execute::recover(deps, info.sender),
        ExecuteMsg::TransferOwnership { new_governor } => {
            execute::transfer_ownership(deps, info.sender, new_governor)
        }
        ExecuteMsg::SetSwapContract { new_contract } => {
            execute::set_swap_contract(deps, info.sender, new_contract)
        }
    }
}

/// Handling contract queries
#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    deps.api.debug("crosschain swaps query invoked");
    match msg {
        QueryMsg::Recoverable { addr } => to_binary(
            &RECOVERY_STATES
                .may_load(deps.storage, &addr)?
                .or(Some(vec![])),
        ),
    }
}

#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    deps.api.debug("crosschain swaps sudo invoked");
    match msg {
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCAck {
            channel,
            sequence,
            ack,
            success,
        }) => ibc_lifecycle::receive_ack(deps, channel, sequence, ack, success),
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCTimeout { channel, sequence }) => {
            ibc_lifecycle::receive_timeout(deps, channel, sequence)
        }
    }
}

#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    deps.api.debug("crosschain swaps reply invoked");
    deps.api
        .debug(&format!("executing crosschain reply: {reply:?}"));
    match MsgReplyID::from_repr(reply.id) {
        Some(MsgReplyID::Swap) => execute::handle_swap_reply(deps, env, reply),
        Some(MsgReplyID::Forward) => execute::handle_forward_reply(deps, reply),
        None => Err(ContractError::InvalidReplyID { id: reply.id }),
    }
}
