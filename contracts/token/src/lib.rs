#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, symbol_short, Vec, Map};

// Storage keys
const CAMPAIGNS: soroban_sdk::Symbol = symbol_short!("CAMPAIGNS");
const CAMPAIGN_COUNT: soroban_sdk::Symbol = symbol_short!("COUNT");
const DONATIONS: soroban_sdk::Symbol = symbol_short!("DONATIONS");
const ADMIN: soroban_sdk::Symbol = symbol_short!("ADMIN");

#[contract]
pub struct ZakatContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CampaignCategory {
    Zakat,
    Pendidikan,
    Kesehatan,
    BencanaAlam,
    UMKM,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    Active,
    Completed,
    Closed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Campaign {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub category: CampaignCategory,
    pub target_amount: i128,
    pub current_amount: i128,
    pub recipient: Address,
    pub status: CampaignStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Donation {
    pub campaign_id: u32,
    pub donor: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub is_anonymous: bool,
}

#[contractimpl]
impl ZakatContract {
    
    // Initialize contract with admin
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&CAMPAIGN_COUNT, &0u32);
    }

    // Create new campaign (only admin)
    pub fn create_campaign(
        env: Env,
        admin: Address,
        title: String,
        description: String,
        category: CampaignCategory,
        target_amount: i128,
        recipient: Address,
    ) -> u32 {
        admin.require_auth();

        // Verify admin
        let stored_admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        if admin != stored_admin {
            panic!("Only admin can create campaigns");
        }

        // Validate
        if target_amount <= 0 {
            panic!("Target amount must be greater than 0");
        }

        // Get current count and increment
        let campaign_id: u32 = env.storage()
            .instance()
            .get(&CAMPAIGN_COUNT)
            .unwrap_or(0);
        
        let new_id = campaign_id + 1;

        // Create campaign
        let campaign = Campaign {
            id: new_id,
            title,
            description,
            category,
            target_amount,
            current_amount: 0,
            recipient,
            status: CampaignStatus::Active,
            created_at: env.ledger().timestamp(),
        };

        // Store campaign
        let key = symbol_short!("CAMP");
        let mut campaigns: Map<u32, Campaign> = env.storage()
            .instance()
            .get(&key)
            .unwrap_or(Map::new(&env));
        
        campaigns.set(new_id, campaign);
        env.storage().instance().set(&key, &campaigns);
        env.storage().instance().set(&CAMPAIGN_COUNT, &new_id);

        new_id
    }

    // Donate to campaign
    pub fn donate(
        env: Env,
        donor: Address,
        campaign_id: u32,
        amount: i128,
        is_anonymous: bool,
    ) {
        donor.require_auth();

        // Validate
        if amount <= 0 {
            panic!("Amount must be greater than 0");
        }

        // Get campaign
        let key = symbol_short!("CAMP");
        let mut campaigns: Map<u32, Campaign> = env.storage()
            .instance()
            .get(&key)
            .unwrap();

        let mut campaign = campaigns.get(campaign_id).unwrap();

        // Check if campaign is active
        if campaign.status != CampaignStatus::Active {
            panic!("Campaign is not active");
        }

        // Update campaign amount
        campaign.current_amount += amount;

        // Check if target reached
        if campaign.current_amount >= campaign.target_amount {
            campaign.status = CampaignStatus::Completed;
        }

        campaigns.set(campaign_id, campaign);
        env.storage().instance().set(&key, &campaigns);

        // Store donation
        let donation = Donation {
            campaign_id,
            donor: donor.clone(),
            amount,
            timestamp: env.ledger().timestamp(),
            is_anonymous,
        };

        let donation_key = symbol_short!("DONS");
        let mut donations: Vec<Donation> = env.storage()
            .instance()
            .get(&donation_key)
            .unwrap_or(Vec::new(&env));
        
        donations.push_back(donation);
        env.storage().instance().set(&donation_key, &donations);

        // Emit event
        env.events().publish(
            (symbol_short!("donate"), campaign_id),
            (donor, amount)
        );
    }

    // Get all campaigns
    pub fn get_campaigns(env: Env) -> Vec<Campaign> {
        let key = symbol_short!("CAMP");
        let campaigns: Map<u32, Campaign> = env.storage()
            .instance()
            .get(&key)
            .unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);
        let count: u32 = env.storage().instance().get(&CAMPAIGN_COUNT).unwrap_or(0);

        for i in 1..=count {
            if let Some(campaign) = campaigns.get(i) {
                result.push_back(campaign);
            }
        }

        result
    }

    // Get campaign by ID
    pub fn get_campaign(env: Env, campaign_id: u32) -> Campaign {
        let key = symbol_short!("CAMP");
        let campaigns: Map<u32, Campaign> = env.storage()
            .instance()
            .get(&key)
            .unwrap();

        campaigns.get(campaign_id).unwrap()
    }

    // Get donations for campaign
    pub fn get_campaign_donations(env: Env, campaign_id: u32) -> Vec<Donation> {
        let donation_key = symbol_short!("DONS");
        let all_donations: Vec<Donation> = env.storage()
            .instance()
            .get(&donation_key)
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);
        
        for i in 0..all_donations.len() {
            let donation = all_donations.get(i).unwrap();
            if donation.campaign_id == campaign_id {
                result.push_back(donation);
            }
        }

        result
    }

    // Get all donations (for admin)
    pub fn get_all_donations(env: Env) -> Vec<Donation> {
        let donation_key = symbol_short!("DONS");
        env.storage()
            .instance()
            .get(&donation_key)
            .unwrap_or(Vec::new(&env))
    }

    // Close campaign (only admin)
    pub fn close_campaign(env: Env, admin: Address, campaign_id: u32) {
        admin.require_auth();

        let stored_admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        if admin != stored_admin {
            panic!("Only admin can close campaigns");
        }

        let key = symbol_short!("CAMP");
        let mut campaigns: Map<u32, Campaign> = env.storage()
            .instance()
            .get(&key)
            .unwrap();

        let mut campaign = campaigns.get(campaign_id).unwrap();
        campaign.status = CampaignStatus::Closed;

        campaigns.set(campaign_id, campaign);
        env.storage().instance().set(&key, &campaigns);
    }

    // Get total donations across all campaigns
    pub fn get_total_donations(env: Env) -> i128 {
        let key = symbol_short!("CAMP");
        let campaigns: Map<u32, Campaign> = env.storage()
            .instance()
            .get(&key)
            .unwrap_or(Map::new(&env));

        let count: u32 = env.storage().instance().get(&CAMPAIGN_COUNT).unwrap_or(0);
        let mut total = 0i128;

        for i in 1..=count {
            if let Some(campaign) = campaigns.get(i) {
                total += campaign.current_amount;
            }
        }

        total
    }

    // Withdraw funds (only recipient of campaign)
    pub fn withdraw(env: Env, recipient: Address, campaign_id: u32) -> i128 {
        recipient.require_auth();

        let key = symbol_short!("CAMP");
        let mut campaigns: Map<u32, Campaign> = env.storage()
            .instance()
            .get(&key)
            .unwrap();

        let mut campaign = campaigns.get(campaign_id).unwrap();

        // Verify recipient
        if recipient != campaign.recipient {
            panic!("Only recipient can withdraw");
        }

        // Check if completed
        if campaign.status != CampaignStatus::Completed {
            panic!("Campaign must be completed to withdraw");
        }

        let amount = campaign.current_amount;
        
        if amount <= 0 {
            panic!("No funds to withdraw");
        }

        // Reset amount and close campaign
        campaign.current_amount = 0;
        campaign.status = CampaignStatus::Closed;

        campaigns.set(campaign_id, campaign);
        env.storage().instance().set(&key, &campaigns);

        amount
    }

    // Get admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN).unwrap()
    }
}

#[cfg(test)]
mod test;