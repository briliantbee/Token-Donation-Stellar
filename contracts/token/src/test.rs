#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    
    env.mock_all_auths();
    client.initialize(&admin);

    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_create_campaign_zakat() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Zakat Fitrah 2025"),
        &String::from_str(&env, "Distribusi zakat untuk fakir miskin"),
        &CampaignCategory::Zakat,
        &50_000_000, // 5 XLM target
        &recipient,
    );

    assert_eq!(campaign_id, 1);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.id, 1);
    assert_eq!(campaign.title, String::from_str(&env, "Zakat Fitrah 2025"));
    assert_eq!(campaign.current_amount, 0);
    assert_eq!(campaign.target_amount, 50_000_000);
    assert_eq!(campaign.status, CampaignStatus::Active);
}

#[test]
fn test_create_campaign_bencana_alam() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Bantu Korban Banjir Jakarta"),
        &String::from_str(&env, "Donasi untuk korban banjir"),
        &CampaignCategory::BencanaAlam,
        &100_000_000, // 10 XLM target
        &recipient,
    );

    assert_eq!(campaign_id, 1);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.category, CampaignCategory::BencanaAlam);
}

#[test]
fn test_create_multiple_campaigns() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    // Create campaign 1
    let id1 = client.create_campaign(
        &admin,
        &String::from_str(&env, "Campaign 1"),
        &String::from_str(&env, "Description 1"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    // Create campaign 2
    let id2 = client.create_campaign(
        &admin,
        &String::from_str(&env, "Campaign 2"),
        &String::from_str(&env, "Description 2"),
        &CampaignCategory::Pendidikan,
        &100_000_000,
        &recipient,
    );

    // Create campaign 3
    let id3 = client.create_campaign(
        &admin,
        &String::from_str(&env, "Campaign 3"),
        &String::from_str(&env, "Description 3"),
        &CampaignCategory::Kesehatan,
        &150_000_000,
        &recipient,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    let campaigns = client.get_campaigns();
    assert_eq!(campaigns.len(), 3);
}

#[test]
fn test_donate_basic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Zakat Fitrah"),
        &String::from_str(&env, "Zakat untuk fakir miskin"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    // Donate 5 XLM
    client.donate(&donor, &campaign_id, &50_000_000, &false);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.current_amount, 50_000_000);
    assert_eq!(campaign.status, CampaignStatus::Completed); // Target reached
}

#[test]
fn test_donate_multiple_donors() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Beasiswa Anak Yatim"),
        &String::from_str(&env, "Dana pendidikan"),
        &CampaignCategory::Pendidikan,
        &100_000_000, // 10 XLM target
        &recipient,
    );

    // Multiple donations
    client.donate(&donor1, &campaign_id, &30_000_000, &false); // 3 XLM
    client.donate(&donor2, &campaign_id, &20_000_000, &true);  // 2 XLM (anonymous)
    client.donate(&donor3, &campaign_id, &40_000_000, &false); // 4 XLM

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.current_amount, 90_000_000); // 9 XLM total
    assert_eq!(campaign.status, CampaignStatus::Active); // Not completed yet

    let donations = client.get_campaign_donations(&campaign_id);
    assert_eq!(donations.len(), 3);
}

#[test]
fn test_donate_anonymous() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Bantuan Kesehatan"),
        &String::from_str(&env, "Bantuan obat-obatan"),
        &CampaignCategory::Kesehatan,
        &100_000_000,
        &recipient,
    );

    // Anonymous donation
    client.donate(&donor, &campaign_id, &30_000_000, &true);

    let donations = client.get_campaign_donations(&campaign_id);
    assert_eq!(donations.len(), 1);
    
    let donation = donations.get(0).unwrap();
    assert_eq!(donation.is_anonymous, true);
    assert_eq!(donation.amount, 30_000_000);
}

#[test]
fn test_campaign_completion() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Modal UMKM"),
        &String::from_str(&env, "Bantuan modal usaha"),
        &CampaignCategory::UMKM,
        &100_000_000, // 10 XLM target
        &recipient,
    );

    // Donate exactly target amount
    client.donate(&donor, &campaign_id, &100_000_000, &false);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.current_amount, 100_000_000);
    assert_eq!(campaign.status, CampaignStatus::Completed);
}

#[test]
fn test_campaign_over_target() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Emergency Fund"),
        &String::from_str(&env, "Dana darurat"),
        &CampaignCategory::BencanaAlam,
        &50_000_000, // 5 XLM target
        &recipient,
    );

    // Donate more than target
    client.donate(&donor, &campaign_id, &70_000_000, &false); // 7 XLM

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.current_amount, 70_000_000);
    assert_eq!(campaign.status, CampaignStatus::Completed); // Still completed
}

#[test]
fn test_get_all_donations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    // Create 2 campaigns
    let campaign_id_1 = client.create_campaign(
        &admin,
        &String::from_str(&env, "Campaign 1"),
        &String::from_str(&env, "Description 1"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    let campaign_id_2 = client.create_campaign(
        &admin,
        &String::from_str(&env, "Campaign 2"),
        &String::from_str(&env, "Description 2"),
        &CampaignCategory::Pendidikan,
        &100_000_000,
        &recipient,
    );

    // Donate to both campaigns
    client.donate(&donor1, &campaign_id_1, &20_000_000, &false);
    client.donate(&donor2, &campaign_id_2, &30_000_000, &false);
    client.donate(&donor1, &campaign_id_2, &15_000_000, &true);

    let all_donations = client.get_all_donations();
    assert_eq!(all_donations.len(), 3);
}

#[test]
fn test_get_total_donations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    // Create 3 campaigns
    let campaign_id_1 = client.create_campaign(
        &admin,
        &String::from_str(&env, "Campaign 1"),
        &String::from_str(&env, "Description 1"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    let campaign_id_2 = client.create_campaign(
        &admin,
        &String::from_str(&env, "Campaign 2"),
        &String::from_str(&env, "Description 2"),
        &CampaignCategory::Pendidikan,
        &100_000_000,
        &recipient,
    );

    let campaign_id_3 = client.create_campaign(
        &admin,
        &String::from_str(&env, "Campaign 3"),
        &String::from_str(&env, "Description 3"),
        &CampaignCategory::Kesehatan,
        &75_000_000,
        &recipient,
    );

    // Donate to all campaigns
    client.donate(&donor, &campaign_id_1, &20_000_000, &false);
    client.donate(&donor, &campaign_id_2, &30_000_000, &false);
    client.donate(&donor, &campaign_id_3, &15_000_000, &false);

    let total = client.get_total_donations();
    assert_eq!(total, 65_000_000); // 6.5 XLM total
}

#[test]
fn test_close_campaign() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Test Campaign"),
        &String::from_str(&env, "Description"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    // Close campaign
    client.close_campaign(&admin, &campaign_id);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Closed);
}

#[test]
#[should_panic(expected = "Campaign is not active")]
fn test_donate_to_closed_campaign() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Test Campaign"),
        &String::from_str(&env, "Description"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    // Close campaign
    client.close_campaign(&admin, &campaign_id);

    // Try to donate to closed campaign (should panic)
    client.donate(&donor, &campaign_id, &10_000_000, &false);
}

#[test]
fn test_withdraw_completed_campaign() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Withdrawal Test"),
        &String::from_str(&env, "Test withdrawal"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    // Donate to complete campaign
    client.donate(&donor, &campaign_id, &50_000_000, &false);

    // Withdraw
    let withdrawn_amount = client.withdraw(&recipient, &campaign_id);
    assert_eq!(withdrawn_amount, 50_000_000);

    // Check campaign is closed and amount is 0
    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.current_amount, 0);
    assert_eq!(campaign.status, CampaignStatus::Closed);
}

#[test]
#[should_panic(expected = "Campaign must be completed to withdraw")]
fn test_withdraw_active_campaign() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Test Campaign"),
        &String::from_str(&env, "Description"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    // Try to withdraw from active campaign (should panic)
    client.withdraw(&recipient, &campaign_id);
}

#[test]
#[should_panic(expected = "Only recipient can withdraw")]
fn test_withdraw_wrong_recipient() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let wrong_recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Test Campaign"),
        &String::from_str(&env, "Description"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    // Complete campaign
    client.donate(&donor, &campaign_id, &50_000_000, &false);

    // Try to withdraw with wrong recipient (should panic)
    client.withdraw(&wrong_recipient, &campaign_id);
}

#[test]
#[should_panic(expected = "Amount must be greater than 0")]
fn test_donate_zero_amount() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let donor = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    let campaign_id = client.create_campaign(
        &admin,
        &String::from_str(&env, "Test Campaign"),
        &String::from_str(&env, "Description"),
        &CampaignCategory::Zakat,
        &50_000_000,
        &recipient,
    );

    // Try to donate 0 amount (should panic)
    client.donate(&donor, &campaign_id, &0, &false);
}

#[test]
#[should_panic(expected = "Target amount must be greater than 0")]
fn test_create_campaign_zero_target() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZakatContract);
    let client = ZakatContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    
    client.initialize(&admin);

    // Try to create campaign with 0 target (should panic)
    client.create_campaign(
        &admin,
        &String::from_str(&env, "Test Campaign"),
        &String::from_str(&env, "Description"),
        &CampaignCategory::Zakat,
        &0,
        &recipient,
    );
}