use cosmwasm_std::{
    from_binary, from_slice,
    testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR},
    to_binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult, WasmQuery,
};
use terra_cosmwasm::TerraQueryWrapper;

use crate::terrand::{LatestRandomResponse, QueryMsg};

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    terrand_querier: TerrandQuerier,
}

#[derive(Clone, Default)]
pub struct TerrandQuerier {}

impl TerrandQuerier {
    pub fn new() -> Self {
        TerrandQuerier {}
    }
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => match from_binary(msg) {
                Ok(QueryMsg::LatestDrand {}) => {
                    SystemResult::Ok(ContractResult::from(to_binary(&LatestRandomResponse {
                        round: 1883848,
                        randomness: to_binary("yTBW2ubloeFa+ZRh08Jt+4jVQHHGMX4s3j8mTYKc3oQ=")
                            .unwrap(),
                        worker: "terra1qqhd8edcc590tdvy7zhhjt62e2gs49wcl22xaq".to_string(),
                    })))
                }
                _ => panic!("DO NOT ENTER HERE"),
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            terrand_querier: TerrandQuerier::default(),
        }
    }

    pub fn with_terrand(&mut self){
        self.terrand_querier = TerrandQuerier::new();
    }
}
