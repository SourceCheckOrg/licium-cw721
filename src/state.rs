use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;
use cw_storage_plus::Map;
use cosmwasm_std::Coin;

/// ISCC data derived from the media asset
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsccData {
    pub token_id: String,
    pub iscc_code: String,
    pub tophash: String,
}

/// Licensing data: price for licensing the token
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Licensing {
    pub token_id: String,
    pub url: String,
    pub price: Coin,
}

/// License transaction
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct License {
    pub token_id: String,
    pub licensee: Addr,
    pub price: Coin,
}

/// maps iscc code to iscc data
pub const ISCC_DATA: Map<&str, IsccData> = Map::new("iscc_data");

/// maps iscc code to token Id
pub const ISCC: Map<&str, String> = Map::new("iscc");

/// maps token id to licensing data
pub const LICENSING: Map<&str, Licensing> = Map::new("license");

/// maps licensee address + token id to license 
pub const LICENSE: Map<(&Addr, &str), License> = Map::new("license");
