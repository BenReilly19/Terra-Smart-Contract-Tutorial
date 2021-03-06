#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{StdResult, Deps, DepsMut, Env, MessageInfo, Response, to_binary, Binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{PoolResponse, ExecuteMsg, InstantiateMsg, QueryMsg};


use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:
";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut _token1 = msg.token1;
    let mut _token2 = msg.token2;

    let state = State {
        token1: _token1,
        token2: _token2,
        amount1: msg.amount1,
        amount2: msg.amount2,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::GetToken1for2 {token1} => get_token1for2(deps, token1),
        ExecuteMsg::GetToken2for1 {token2} => get_token2for1(deps, token2),
    }
}

pub fn get_token1for2(deps: DepsMut, amount: i32) -> Result<Response, ContractError> {

        let exchange_rate = 1;
        let pool_amount = amount;
        let return_amount = pool_amount * exchange_rate;
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if return_amount > state.amount2 {
            return Err(ContractError::Unauthorized {})
        }  
        state.amount1 += amount;
        state.amount2 = state.amount2 - return_amount;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "GetToken1for2"))
}

pub fn get_token2for1(deps: DepsMut, amount: i32) -> Result<Response, ContractError> {
        let exchange_rate = 1;
        let pool_amount = amount;
        let return_amount = pool_amount * exchange_rate;
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if return_amount > state.amount1 {
                return Err(ContractError::Unauthorized {})
            };
            state.amount2 += amount;
            state.amount1 = state.amount1 - return_amount;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("method", "GetToken2for1"))
    }


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPool {} => to_binary(&get_pool(deps)?),
    }
}

pub fn get_pool(deps: Deps) -> StdResult<PoolResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(PoolResponse { amount1: state.amount1, amount2: state.amount2, token1: state.token1, token2: state.token2 })
    }


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { 
            token1: "token1".to_string(),
            token2: "token2".to_string(),
            amount1: 50,
            amount2: 50
        };

        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPool{}).unwrap();
        let value: PoolResponse = from_binary(&res).unwrap();
        assert_eq!("token1", value.token1);
        assert_eq!("token2", value.token2);
        assert_eq!(50, value.amount1);
        assert_eq!(50, value.amount2);
    }

    #[test]
    fn trade2for1 () {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { 
            token1: "token1".to_string(),
            token2: "token2".to_string(),
            amount1: 50,
            amount2: 50
        };
        
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::GetToken2for1 { token2: 1 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should Increase token2 amount by 1, reduce token 1 amount by 1.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPool {}).unwrap();
        let value: PoolResponse = from_binary(&res).unwrap();
        assert_eq!("token1", value.token1);
        assert_eq!("token2", value.token2);
        assert_eq!(49, value.amount1);
        assert_eq!(51, value.amount2);
    }

    #[test]
    fn trade1for2 () {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { 
            token1: "token1".to_string(),
            token2: "token2".to_string(),
            amount1: 50,
            amount2: 50
        };
        
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::GetToken1for2 { token1: 1 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should Increase token2 amount by 1, reduce token 1 amount by 1.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPool {}).unwrap();
        let value: PoolResponse = from_binary(&res).unwrap();
        assert_eq!("token1", value.token1);
        assert_eq!("token2", value.token2);
        assert_eq!(51, value.amount1);
        assert_eq!(49, value.amount2);
    }
}