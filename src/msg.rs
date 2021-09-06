use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, Coin};
use cw721::Expiration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    /// Unique ID of the NFT
    pub token_id: String,
    /// ISCC code 
    pub iscc_code: String,
    /// Tophash
    pub tophash: String,
    /// The owner of the newly minter NFT
    pub owner: String,
    /// Identifies the asset to which this NFT represents
    pub name: String,
    /// Describes the asset to which this NFT represents (may be empty)
    pub description: String,
    /// A URI pointing to an image representing the asset
    pub image: String,
    /// License url
    pub license_url: String,
    /// Price to license 
    pub license_price: Coin,

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg),
    
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    
    /// Buy a license
    License {
        token_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return the owner of the given token, error if token does not exist
    /// Return type: OwnerOfResponse
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    /// List all operators that can access all of the owner's tokens
    /// Return type: `ApprovedForAllResponse`
    ApprovedForAll {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Total number of tokens issued
    NumTokens {},

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract: `ContractInfoResponse`
    ContractInfo {},

    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract: `NftInfoResponse`
    NftInfo {
        token_id: String,
    },

    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients: `AllNftInfo`
    AllNftInfo {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    /// Return type: TokensResponse.
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    /// Return type: TokensResponse.
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Resolve token by ISCC code
    /// Return type: 
    GetByIsccCode {
        iscc_code: String,
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenResponse {
    pub token_id: String,
    /// Describes the asset to which this NFT represents
    pub name: String,
    pub description: Option<String>,
    /// "A URI pointing to a resource with mime type image/* representing the asset to which this
    /// NFT represents. Consider making any images at a width between 320 and 1080 pixels and aspect
    /// ratio between 1.91:1 and 4:5 inclusive.
    /// TODO: Use https://docs.rs/url_serde for type-safety
    pub image: Option<String>,
    pub license_url: String,
    pub license_price: Coin,
}
