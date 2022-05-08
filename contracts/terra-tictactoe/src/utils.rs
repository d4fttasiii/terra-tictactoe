use cosmwasm_std::{to_binary, Binary, QuerierWrapper, QueryRequest, StdResult, WasmQuery};

use crate::terrand::{LatestRandomResponse, QueryMsg as TerrandQueryMsg};

pub fn get_randomness(querier: QuerierWrapper, terrand_oracle_address: String) -> StdResult<Binary> {
    let response: LatestRandomResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: terrand_oracle_address,
        msg: to_binary(&TerrandQueryMsg::LatestDrand {})?,
    }))?;

    Ok(response.randomness)
}

pub fn generate_random_u8(randomness: &Binary, round: usize, max: u8) -> u8 {
    let randomness_vector = randomness.to_vec();
    let mut random_array = [0u8; 2];
    random_array[0] = randomness_vector[round];
    random_array[1] = randomness_vector[round + 1];
    let random_big_number = u16::from_be_bytes(random_array);
    let random_ranged_number = random_big_number.wrapping_rem_euclid(max.into()) as u8;

    random_ranged_number
}
