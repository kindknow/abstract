mod common;

use abstract_core::manager::SubAccountIdsResponse;
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::*;
use common::*;
use cosmwasm_std::{wasm_execute, Addr};
use cw_orch::contract::Deploy;
use cw_orch::prelude::*;
// use cw_multi_test::StakingInfo;

#[test]
fn creating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let sub_accounts = account.manager.sub_account_ids(None, None)?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            // only one sub-account and it should be account_id 2
            sub_accounts: vec![2]
        }
    );
    Ok(())
}

#[test]
fn updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts = get_account_contracts(&deployment.version_control, Some(2));
    let new_desc = "new desc";
    account_contracts
        .0
        .update_info(Some(new_desc.to_string()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        account_contracts.0.info()?.info.description
    );

    Ok(())
}

#[test]
fn proxy_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let proxy_address = account.proxy.address()?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let (sub_manager, _sub_proxy) = get_account_contracts(&deployment.version_control, Some(2));
    let new_desc = "new desc";

    // We call as the proxy, it should also be possible
    sub_manager
        .call_as(&proxy_address)
        .update_info(Some(new_desc.to_owned()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        sub_manager.info()?.info.description
    );

    Ok(())
}

#[test]
fn recursive_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts = get_account_contracts(&deployment.version_control, Some(2));

    // We call as the manager, it should also be possible
    account_contracts.0.create_sub_account(
        vec![],
        "My subsubaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let account_contracts = get_account_contracts(&deployment.version_control, Some(3));
    let new_desc = "new desc";

    account_contracts
        .0
        .call_as(&sender)
        .update_info(Some(new_desc.to_string()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        account_contracts.0.info()?.info.description
    );

    Ok(())
}

#[test]
fn installed_app_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let first_proxy_addr = account.proxy.address()?;

    let mock_app = Addr::unchecked("mock_app");
    account
        .proxy
        .call_as(&account.manager.address()?)
        .add_module(mock_app.to_string())?;

    let (sub_manager, _sub_proxy) = get_account_contracts(&deployment.version_control, Some(2));
    let new_desc = "new desc";

    // recover address on first proxy
    account.proxy.set_address(&first_proxy_addr);
    // adding mock_app to whitelist on proxy

    // We call as installed app of the owner-proxy, it should also be possible
    account
        .proxy
        .call_as(&mock_app)
        .module_action(vec![wasm_execute(
            sub_manager.addr_str()?,
            &abstract_core::manager::ExecuteMsg::UpdateInfo {
                name: None,
                description: Some(new_desc.to_owned()),
                link: None,
            },
            vec![],
        )?
        .into()])?;

    assert_eq!(
        Some(new_desc.to_string()),
        sub_manager.info()?.info.description
    );

    Ok(())
}

#[test]
fn sub_account_move_ownership() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let new_owner = Addr::unchecked("new_owner");
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    // Store manager address, it will be used for querying
    let manager_addr = account.manager.address()?;

    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let sub_accounts = account.manager.sub_account_ids(None, None)?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            // only one sub-account and it should be account_id 2
            sub_accounts: vec![2]
        }
    );

    let sub_account = AbstractAccount::new(&deployment, Some(2));
    sub_account.manager.set_owner(GovernanceDetails::Monarchy {
        monarch: new_owner.to_string(),
    })?;

    // Make sure it's not updated until claimed
    let sub_accounts: SubAccountIdsResponse = chain.query(
        &abstract_core::manager::QueryMsg::SubAccountIds {
            start_after: None,
            limit: None,
        },
        &manager_addr,
    )?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            sub_accounts: vec![2]
        }
    );

    // Claim ownership
    sub_account.manager.call_as(&new_owner).execute(
        &abstract_core::manager::ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership),
        None,
    )?;
    let account = AbstractAccount::new(&deployment, Some(1));

    // After claim it's updated
    let sub_accounts = account.manager.sub_account_ids(None, None)?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            sub_accounts: vec![]
        }
    );

    Ok(())
}

#[test]
fn account_move_ownership_to_sub_account() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let sub_account = AbstractAccount::new(&deployment, Some(2));
    let sub_manager_addr = sub_account.manager.address()?;
    let sub_proxy_addr = sub_account.proxy.address()?;

    let new_account = create_default_account(&deployment.account_factory)?;
    let new_governance = GovernanceDetails::SubAccount {
        manager: sub_manager_addr.to_string(),
        proxy: sub_proxy_addr.to_string(),
    };
    new_account.manager.set_owner(new_governance.clone())?;
    let new_account_manager = new_account.manager.address()?;

    let sub_account = AbstractAccount::new(&deployment, Some(2));
    let mock_module = Addr::unchecked("mock_module");
    sub_account
        .proxy
        .call_as(&sub_manager_addr)
        .add_module(mock_module.to_string())?;
    sub_account
        .proxy
        .call_as(&mock_module)
        .module_action(vec![wasm_execute(
            new_account_manager,
            &abstract_core::manager::ExecuteMsg::UpdateOwnership(
                cw_ownable::Action::AcceptOwnership,
            ),
            vec![],
        )?
        .into()])?;

    // sub-accounts state updated
    let sub_ids = sub_account.manager.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![3]);

    // owner of new_account updated
    let new_account = AbstractAccount::new(&deployment, Some(3));
    let info = new_account.manager.info()?.info;
    assert_eq!(new_governance, info.governance_details.into());

    Ok(())
}

#[test]
fn sub_account_move_ownership_to_sub_account() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let sub_account = AbstractAccount::new(&deployment, Some(2));
    let sub_manager_addr = sub_account.manager.address()?;
    let sub_proxy_addr = sub_account.proxy.address()?;

    let new_account = create_default_account(&deployment.account_factory)?;
    new_account.manager.create_sub_account(
        vec![],
        "My second subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;

    // sub-accounts state updated
    let sub_ids = new_account.manager.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![4]);

    let new_account_sub_account = AbstractAccount::new(&deployment, Some(4));
    let new_governance = GovernanceDetails::SubAccount {
        manager: sub_manager_addr.to_string(),
        proxy: sub_proxy_addr.to_string(),
    };
    new_account_sub_account
        .manager
        .set_owner(new_governance.clone())?;
    let new_account_sub_account_manager = new_account_sub_account.manager.address()?;

    let sub_account = AbstractAccount::new(&deployment, Some(2));
    let mock_module = Addr::unchecked("mock_module");
    sub_account
        .proxy
        .call_as(&sub_manager_addr)
        .add_module(mock_module.to_string())?;
    sub_account
        .proxy
        .call_as(&mock_module)
        .module_action(vec![wasm_execute(
            new_account_sub_account_manager,
            &abstract_core::manager::ExecuteMsg::UpdateOwnership(
                cw_ownable::Action::AcceptOwnership,
            ),
            vec![],
        )?
        .into()])?;

    // sub-accounts state updated
    let sub_ids = sub_account.manager.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![4]);
    let new_account = AbstractAccount::new(&deployment, Some(3));
    // removed from the previous owner as well
    let sub_ids = new_account.manager.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, Vec::<u32>::new());

    let new_account_sub_account = AbstractAccount::new(&deployment, Some(4));
    let info = new_account_sub_account.manager.info()?.info;
    assert_eq!(new_governance, info.governance_details.into());
    Ok(())
}

#[test]
fn account_move_ownership_to_falsy_sub_account() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let proxy_addr = account.proxy.address()?;

    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let sub_account = AbstractAccount::new(&deployment, Some(2));
    let sub_manager_addr = sub_account.manager.address()?;

    let new_account = create_default_account(&deployment.account_factory)?;
    // proxy and manager of different accounts
    let new_governance = GovernanceDetails::SubAccount {
        manager: sub_manager_addr.to_string(),
        proxy: proxy_addr.to_string(),
    };
    let err = new_account.manager.set_owner(new_governance).unwrap_err();
    let err = err.root().to_string();
    assert!(err.contains("manager and proxy has different account ids"));
    Ok(())
}