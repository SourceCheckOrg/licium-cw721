#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, BlockInfo, Deps, DepsMut, Env, MessageInfo, Order, Pair, 
    Response, StdError, StdResult,
};

use cw0::maybe_addr;
use cw2::set_contract_version;
use cw721::{
    AllNftInfoResponse, ApprovedForAllResponse, ContractInfoResponse, Expiration, NftInfoResponse, 
    NumTokensResponse, OwnerOfResponse, TokensResponse,
};

use cw721_base::contract::{
    execute_approve, execute_revoke, execute_approve_all, execute_revoke_all, execute_transfer_nft,
    execute_send_nft
};
use cw721_base::ContractError; // TODO use custom errors instead
use cw721_base::state::{Approval, CONTRACT_INFO, increment_tokens, num_tokens, OPERATORS, TokenInfo, tokens};
use cw_storage_plus::Bound;

use crate::msg::{InstantiateMsg, ExecuteMsg, MintMsg, QueryMsg, TokenResponse};
use crate::state::{IsccData, ISCC_DATA, ISCC, License, LICENSE, Licensing, LICENSING };

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:licium-cw721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// used for limiting queries
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let contract_info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };
    CONTRACT_INFO.save(deps.storage, &contract_info)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint(msg) => execute_mint(deps, env, info, msg),
        ExecuteMsg::License {
            token_id,
        } => execute_licensing(deps, info, token_id),
        ExecuteMsg::Approve { 
            spender,
            token_id, 
            expires,
        } => execute_approve(deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke { 
            spender, 
            token_id,
        } => execute_revoke(deps, env, info, spender, token_id),
        ExecuteMsg::ApproveAll { 
            operator, 
            expires,
        } => execute_approve_all(deps, env, info, operator, expires),
        ExecuteMsg::RevokeAll { 
            operator 
        } => execute_revoke_all(deps, env, info, operator),
        ExecuteMsg::TransferNft { 
            recipient, 
            token_id, 
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft { 
            contract, 
            token_id, 
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    // create the token
    let token = TokenInfo {
        name: msg.name.clone(),
        description: msg.description.clone(),
        image: Some(msg.image.clone()),
        owner: info.sender,
        approvals: vec![],
    };
    tokens().update(
        deps.storage, 
        &msg.token_id, 
        |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token),
        }
    )?;
    
    // update tokens count
    increment_tokens(deps.storage)?;

    // store iscc data related to content id 
    let iscc_data = IsccData {
        token_id: msg.token_id.clone(),
        meta_id: msg.meta_id.clone(),
        content_id: msg.content_id.clone(),
        data_id: msg.data_id.clone(),
        instance_id: msg.instance_id.clone(),
        tophash: msg.tophash.clone(),
    };
    ISCC_DATA.save(deps.storage, &msg.content_id, &iscc_data)?;

    // associate iscc content id with token
    ISCC.update(
        deps.storage, 
        &msg.content_id, 
        | old | match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(msg.token_id.clone())
        }
    )?;

    // store licensing data
    let licensing = Licensing {
        token_id: msg.token_id.clone(),
        url: msg.license_url.clone(),
        price: msg.license_price,
    };
    LICENSING.save(deps.storage, &msg.token_id, &licensing)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("token_id", msg.token_id)
        .add_attribute("name", msg.name)
        .add_attribute("content_id", msg.content_id)
        .add_attribute("owner", msg.owner)
    )
}

pub fn execute_licensing(
    deps: DepsMut,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    // load licensing info 
    let licensing = LICENSING.load(deps.storage, &token_id)?;

    // extract coins sent
    // TODO handle multiple coins
    let coins_sent = &info.funds[0];
    
    // check if the coins sent are the ones accepted for licensing and if the amount sent is enough
    if coins_sent.denom != licensing.price.denom ||  coins_sent.amount < licensing.price.amount {
        return Err(ContractError::Unauthorized{}); // TODO throw custom error
    }

    // load token info and send funds to token owner
    let token_info = tokens().load(deps.storage, &token_id)?;
    let send_funds_msg = BankMsg::Send {
        to_address: token_info.owner.to_string(),
        amount: vec![licensing.price],
    };

    // save license \transaction
    let license = License {
        token_id: token_id.clone(),
        price: coins_sent.clone(),
        licensee: info.sender.clone(),
    };
    LICENSE.save(deps.storage, (&info.sender, &token_id), &license)?;

    Ok(Response::new()
        .add_message(send_funds_msg)
        .add_attribute("action", "license")
        .add_attribute("token_id", token_id)
        .add_attribute("price", info.funds[0].amount)
        .add_attribute("licensee", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::NftInfo { 
            token_id 
        } => to_binary(&query_nft_info(
            deps, 
            token_id
        )?),
        QueryMsg::OwnerOf { 
            token_id, 
            include_expired 
        } => to_binary(&query_owner_of(
            deps, 
            env, 
            token_id, 
            include_expired.unwrap_or(false),
        )?),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => to_binary(&query_all_nft_info(
            deps,
            env,
            token_id,
            include_expired.unwrap_or(false),
        )?),
        QueryMsg::ApprovedForAll {
            owner,
            include_expired,
            start_after,
            limit,
        } => to_binary(&query_all_approvals(
            deps,
            env,
            owner,
            include_expired.unwrap_or(false),
            start_after,
            limit,
        )?),
        QueryMsg::NumTokens {} => to_binary(&query_num_tokens(deps)?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => to_binary(&query_tokens(
            deps, 
            owner, 
            start_after, 
            limit
        )?),
        QueryMsg::AllTokens { 
            start_after, 
            limit 
        } => {
            to_binary(&query_all_tokens(deps, start_after, limit)?)
        },
        QueryMsg::GetByContentId {
            content_id
        } => {
            to_binary(&get_by_iscc_code(deps, content_id)?)
        }
    }
}

fn get_by_iscc_code(deps: Deps, content_id: String) -> StdResult<Option<TokenResponse>> {
    let token_id_result = ISCC.load(deps.storage, &content_id);

    match token_id_result {
        Ok(token_id) => {
            let token_info = tokens().load(deps.storage, &token_id)?;
            let iscc_data = ISCC_DATA.load(deps.storage, &content_id)?;
            let licensing = LICENSING.load(deps.storage, &token_id)?;
            
            Ok(Some(TokenResponse {
                token_id: token_id,
                owner: token_info.owner,
                name: token_info.name,
                description: Some(token_info.description),
                image: token_info.image,
                meta_id: iscc_data.meta_id,
                content_id: iscc_data.content_id,
                data_id: iscc_data.data_id,
                instance_id: iscc_data.instance_id,
                license_url: licensing.url,
                license_price: licensing.price,
            }))
        },
        Err(_) => Ok(None)
    }
}

fn query_contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    CONTRACT_INFO.load(deps.storage)
}

fn query_nft_info(deps: Deps, token_id: String) -> StdResult<NftInfoResponse> {
    let info = tokens().load(deps.storage, &token_id)?;
    Ok(NftInfoResponse {
        name: info.name,
        description: info.description,
        image: info.image,
    })
}

fn query_owner_of(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<OwnerOfResponse> {
    let info = tokens().load(deps.storage, &token_id)?;
    Ok(OwnerOfResponse {
        owner: info.owner.to_string(),
        approvals: humanize_approvals(&env.block, &info, include_expired),
    })
}

fn query_all_nft_info(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<AllNftInfoResponse> {
    let info = tokens().load(deps.storage, &token_id)?;
    Ok(AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info, include_expired),
        },
        info: NftInfoResponse {
            name: info.name,
            description: info.description,
            image: info.image,
        },
    })
}

fn query_all_approvals(
    deps: Deps,
    env: Env,
    owner: String,
    include_expired: bool,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ApprovedForAllResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(|addr| Bound::exclusive(addr.as_ref()));

    let owner_addr = deps.api.addr_validate(&owner)?;
    let res: StdResult<Vec<_>> = OPERATORS
        .prefix(&owner_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|r| include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block))
        .take(limit)
        .map(parse_approval)
        .collect();
    Ok(ApprovedForAllResponse { operators: res? })
}

fn parse_approval(item: StdResult<Pair<Expiration>>) -> StdResult<cw721::Approval> {
    item.and_then(|(k, expires)| {
        let spender = String::from_utf8(k)?;
        Ok(cw721::Approval { spender, expires })
    })
}

fn humanize_approvals(
    block: &BlockInfo,
    info: &TokenInfo,
    include_expired: bool,
) -> Vec<cw721::Approval> {
    info.approvals
        .iter()
        .filter(|apr| include_expired || !apr.is_expired(block))
        .map(humanize_approval)
        .collect()
}

fn humanize_approval(approval: &Approval) -> cw721::Approval {
    cw721::Approval {
        spender: approval.spender.to_string(),
        expires: approval.expires,
    }
}

fn query_num_tokens(deps: Deps) -> StdResult<NumTokensResponse> {
    let count = num_tokens(deps.storage)?;
    Ok(NumTokensResponse { count })
}

fn query_tokens(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let owner_addr = deps.api.addr_validate(&owner)?;
    let pks: Vec<_> = tokens()
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let res: Result<Vec<_>, _> = pks.iter().map(|v| String::from_utf8(v.to_vec())).collect();
    let tokens = res.map_err(StdError::invalid_utf8)?;
    Ok(TokensResponse { tokens })
}

fn query_all_tokens(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(|addr| Bound::exclusive(addr.as_ref()));

    let tokens: StdResult<Vec<String>> = tokens()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
        .collect();
    Ok(TokensResponse { tokens: tokens? })
}

// TODO implement tests