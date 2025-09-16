// SPDX-License-Identifier: MIT
#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, symbol_short};

fn create_contract() -> (Env, LendingContract, Address, Address) {
    let env = Env::default();
    let contract = LendingContract;
    let owner = Address::generate(&env);
    let borrower = Address::generate(&env);
    
    contract.__constructor(&env, owner.clone());
    
    (env, contract, owner, borrower)
}

#[test]
fn test_constructor() {
    let (env, contract, owner, _) = create_contract();
    
    // Verificar se o owner foi definido corretamente
    assert!(contract.owner(&env) == owner);
}

#[test]
fn test_mint_nft() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner);
    
    // Verificar se o NFT foi criado
    assert_eq!(contract.owner_of(&env, token_id), Some(borrower));
}

#[test]
fn test_create_loan() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    let loan_amount = 1000i128;
    let interest_rate = 500u32; // 5%
    let duration_days = 30u32;
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner);
    
    // Criar empréstimo
    let loan_id = contract.create_loan(
        &env,
        borrower.clone(),
        token_id,
        loan_amount,
        interest_rate,
        duration_days,
        borrower.clone()
    );
    
    // Verificar se o empréstimo foi criado
    assert_eq!(loan_id, 1);
    
    // Verificar se o token é usado como colateral
    assert!(contract.is_collateral(&env, token_id));
    
    // Verificar informações do empréstimo
    let loan_info = contract.get_loan_info(&env, loan_id);
    assert_eq!(loan_info.borrower, borrower);
    assert_eq!(loan_info.collateral_token_id, token_id);
    assert_eq!(loan_info.loan_amount, loan_amount);
    assert_eq!(loan_info.interest_rate, interest_rate);
    assert_eq!(loan_info.duration_days, duration_days);
    assert_eq!(loan_info.status, LoanStatus::Active);
    assert_eq!(loan_info.repaid_amount, 0);
}

#[test]
fn test_repay_loan() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    let loan_amount = 1000i128;
    let interest_rate = 500u32; // 5%
    let duration_days = 30u32;
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner);
    
    // Criar empréstimo
    let loan_id = contract.create_loan(
        &env,
        borrower.clone(),
        token_id,
        loan_amount,
        interest_rate,
        duration_days,
        borrower.clone()
    );
    
    // Pagar parte do empréstimo
    let partial_payment = 500i128;
    contract.repay_loan(&env, loan_id, partial_payment, borrower.clone());
    
    // Verificar se o pagamento foi registrado
    let loan_info = contract.get_loan_info(&env, loan_id);
    assert_eq!(loan_info.repaid_amount, partial_payment);
    assert_eq!(loan_info.status, LoanStatus::Active); // Ainda ativo pois não foi totalmente pago
    
    // Pagar o restante
    let remaining = loan_amount - partial_payment;
    contract.repay_loan(&env, loan_id, remaining, borrower.clone());
    
    // Verificar se o empréstimo foi totalmente pago
    let loan_info = contract.get_loan_info(&env, loan_id);
    assert_eq!(loan_info.status, LoanStatus::Repaid);
    assert!(!contract.is_collateral(&env, token_id)); // Colateral deve ser liberado
}

#[test]
fn test_liquidate_loan() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    let loan_amount = 1000i128;
    let interest_rate = 500u32; // 5%
    let duration_days = 1u32; // 1 dia para facilitar o teste
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner.clone());
    
    // Criar empréstimo
    let loan_id = contract.create_loan(
        &env,
        borrower.clone(),
        token_id,
        loan_amount,
        interest_rate,
        duration_days,
        borrower.clone()
    );
    
    // Avançar o tempo para simular vencimento
    env.ledger().set_timestamp(env.ledger().timestamp() + (2 * 24 * 60 * 60)); // 2 dias depois
    
    // Liquidar empréstimo
    contract.liquidate_loan(&env, loan_id, owner);
    
    // Verificar se o empréstimo foi liquidado
    let loan_info = contract.get_loan_info(&env, loan_id);
    assert_eq!(loan_info.status, LoanStatus::Liquidated);
    
    // Verificar se o NFT foi transferido para o owner (liquidator)
    assert_eq!(contract.owner_of(&env, token_id), Some(owner));
}

#[test]
fn test_calculate_interest() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    let loan_amount = 1000i128;
    let interest_rate = 1000u32; // 10%
    let duration_days = 365u32; // 1 ano
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner);
    
    // Criar empréstimo
    let loan_id = contract.create_loan(
        &env,
        borrower.clone(),
        token_id,
        loan_amount,
        interest_rate,
        duration_days,
        borrower.clone()
    );
    
    // Avançar o tempo para 1 ano
    env.ledger().set_timestamp(env.ledger().timestamp() + (365 * 24 * 60 * 60));
    
    // Calcular juros
    let loan_info = contract.get_loan_info(&env, loan_id);
    let interest = contract.calculate_interest(&env, &loan_info);
    
    // Juros esperados: 1000 * 10 * 365 / (100 * 365) = 100
    assert_eq!(interest, 100);
}

#[test]
fn test_get_user_loans() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id1 = 1u32;
    let token_id2 = 2u32;
    
    // Mint NFTs para o borrower
    contract.mint(&env, borrower.clone(), token_id1, owner);
    contract.mint(&env, borrower.clone(), token_id2, owner);
    
    // Criar dois empréstimos
    let loan_id1 = contract.create_loan(
        &env,
        borrower.clone(),
        token_id1,
        1000i128,
        500u32,
        30u32,
        borrower.clone()
    );
    
    let loan_id2 = contract.create_loan(
        &env,
        borrower.clone(),
        token_id2,
        2000i128,
        600u32,
        60u32,
        borrower.clone()
    );
    
    // Verificar empréstimos do usuário
    let user_loans = contract.get_user_loans(&env, borrower.clone());
    assert_eq!(user_loans.len(), 2);
    assert!(user_loans.contains(loan_id1));
    assert!(user_loans.contains(loan_id2));
}

#[test]
#[should_panic(expected = "Not owner of collateral")]
fn test_create_loan_not_owner() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    let other_borrower = Address::generate(&env);
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner);
    
    // Tentar criar empréstimo com outro usuário
    contract.create_loan(
        &env,
        other_borrower,
        token_id,
        1000i128,
        500u32,
        30u32,
        other_borrower
    );
}

#[test]
#[should_panic(expected = "Token already used as collateral")]
fn test_create_loan_already_collateral() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner);
    
    // Criar primeiro empréstimo
    contract.create_loan(
        &env,
        borrower.clone(),
        token_id,
        1000i128,
        500u32,
        30u32,
        borrower.clone()
    );
    
    // Tentar criar segundo empréstimo com o mesmo token
    contract.create_loan(
        &env,
        borrower.clone(),
        token_id,
        2000i128,
        600u32,
        60u32,
        borrower.clone()
    );
}

#[test]
#[should_panic(expected = "Not the borrower")]
fn test_repay_loan_not_borrower() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    let other_user = Address::generate(&env);
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner);
    
    // Criar empréstimo
    let loan_id = contract.create_loan(
        &env,
        borrower.clone(),
        token_id,
        1000i128,
        500u32,
        30u32,
        borrower.clone()
    );
    
    // Tentar pagar com outro usuário
    contract.repay_loan(&env, loan_id, 500i128, other_user);
}

#[test]
#[should_panic(expected = "Loan not yet expired")]
fn test_liquidate_loan_not_expired() {
    let (env, contract, owner, borrower) = create_contract();
    let token_id = 1u32;
    let loan_amount = 1000i128;
    let interest_rate = 500u32;
    let duration_days = 30u32; // 30 dias
    
    // Mint um NFT para o borrower
    contract.mint(&env, borrower.clone(), token_id, owner.clone());
    
    // Criar empréstimo
    let loan_id = contract.create_loan(
        &env,
        borrower.clone(),
        token_id,
        loan_amount,
        interest_rate,
        duration_days,
        borrower.clone()
    );
    
    // Tentar liquidar antes do vencimento
    contract.liquidate_loan(&env, loan_id, owner);
}
