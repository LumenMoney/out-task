#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
                    Uint128, QueryRequest, BankQuery, BalanceResponse,
                    CosmosMsg, BankMsg, Coin};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ UserAndCompanyResponse ,UserBalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:interview-task";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let company = deps.api.addr_validate(&msg.company)?;
    let user = deps.api.addr_validate(&msg.user)?;
    let state = State {
        user: user,
        company: company,
        owner: info.sender.clone()
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("user", msg.user)
        .add_attribute("company", msg.company)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => try_deposit(),
        ExecuteMsg::Withdraw{amount} => try_withdraw(deps, info, env, amount)
        
    }
}

pub fn try_withdraw(deps: DepsMut, info: MessageInfo, env: Env, amount: u64) -> Result<Response, ContractError>{
    let sender = info.sender.clone();
    let state = STATE.load(deps.storage)?;
    
    //Only proceed if the sender matches User or Company
    if sender == state.user || sender == state.company {

        let mut to_company = false;
        let to_account;
    
        
        // Set to_company true for transfer of funds to company
        if sender == state.company{
            to_company = true;
        }

        // Get contract balance
        let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
            address: env.contract.address.to_string(),
            denom: "uusd".to_string(),
        }))?;

        // Do not proceed if withdrawl amount is greater than balance
        if Uint128::from(amount) > balance.amount.amount {
            return Err(ContractError::WithdrwalExceedsBalance {}); 
        }

        // Set the adress for the transfer
        if to_company {
            to_account = state.company;
        }else{
            to_account = sender;
        }

        Ok(Response::new()
        .add_message(
            CosmosMsg::Bank(BankMsg::Send{
                to_address: to_account.to_string(),
                amount: vec![
                    Coin{
                        denom: "uusd".to_string(), 
                        amount: Uint128::from(amount)
                    }]
                
            })
        ))

    }else{
        return Err(ContractError::Unauthorized{})
    }
    
}

pub fn try_deposit() -> Result<Response, ContractError>{
    Ok(Response::new())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance{} => to_binary(&query_balance(deps, env)?),
        QueryMsg::GetUserAndCompany {} => to_binary(&query_user_and_company(deps)?),
    }
}

fn query_user_and_company(deps: Deps) -> StdResult<UserAndCompanyResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(UserAndCompanyResponse{
        user: state.user.to_string(),
        company: state.company.to_string(),

    })
}

fn query_balance(deps: Deps, env: Env) -> StdResult<UserBalanceResponse>{
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: env.contract.address.to_string(),
        denom: "uusd".to_string(),
    }))?;
    Ok(UserBalanceResponse{balance: balance.amount.amount.u128() as u64})

}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let company = "terra1e8ryd9ezefuucd4mje33zdms9m2s90m57878v9".to_string();
        let user= "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8".to_string();

        let msg = InstantiateMsg { company: company.clone(), user: user.clone() };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetUserAndCompany {}).unwrap();
        let value: UserAndCompanyResponse = from_binary(&res).unwrap();
        assert_eq!(company, value.company);
        assert_eq!(user, value.user);
    }

    #[test]
    fn deposit() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let company = "terra1e8ryd9ezefuucd4mje33zdms9m2s90m57878v9".to_string();
        let user= "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8".to_string();

        let msg = InstantiateMsg { company, user: user.clone() };
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Deposit UST
        let info = mock_info(&user.clone(), &coins(2, "uusd"));
        let msg = ExecuteMsg::Deposit {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // In Test Environment, we can only set contract's balance 
        // by mock_dependencies. So, we would have to deploy the contract
        // to test it.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetBalance {}).unwrap();
        let value: UserBalanceResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.balance);
    }

    #[test]
    fn withdraw() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let company = "terra1e8ryd9ezefuucd4mje33zdms9m2s90m57878v9".to_string();
        let user= "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8".to_string();
        let not_allowed_user= "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd6".to_string();

        let msg = InstantiateMsg { company, user: user.clone() };
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Withdrawing UST by a not allowed User results
        // in unauthorized error
        let info = mock_info(&not_allowed_user.clone(), &coins(2, "uusd"));
        let msg = ExecuteMsg::Withdraw { amount: 2};
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        match res {
            Err(ContractError::Unauthorized{}) => {},
            _ => panic!("Must return unauthorized error")
        }
        
    }


}
