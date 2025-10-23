#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, Env, Address, Symbol, symbol_short, Vec, Map, String,
};

#[derive(Clone)]
#[contracttype]
pub enum CampaignStatus {
    Active,
    Completed,
    Closed,
}

#[derive(Clone)]
#[contracttype]
pub struct Campaign {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub category: String,
    pub target_amount: i128,
    pub current_amount: i128,
    pub recipient: Address,
    pub status: CampaignStatus,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Donation {
    pub donor: Address,
    pub amount: i128,
    pub timestamp: u64,
}

// 🔹 Keys untuk penyimpanan
const CAMPAIGNS: Symbol = symbol_short!("CAMPAIGNS");
const DONATIONS: Symbol = symbol_short!("DONATIONS");
const CAMPAIGN_COUNT: Symbol = symbol_short!("COUNT");

#[contract]
pub struct ZakatContract;

#[contractimpl]
impl ZakatContract {
    // ----------------------------------------------------
    // Inisialisasi kontrak (set counter awal)
    // ----------------------------------------------------
    pub fn initialize(env: Env) {
        if env.storage().instance().has(&CAMPAIGN_COUNT) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&CAMPAIGN_COUNT, &0u32);
    }

    // ----------------------------------------------------
    // Membuat campaign baru
    // ----------------------------------------------------
    pub fn create_campaign(
        env: Env,
        title: String,
        description: String,
        category: String,
        target_amount: i128,
        recipient: Address,
    ) -> u32 {
        recipient.require_auth();

        let mut count: u32 = env
            .storage()
            .instance()
            .get(&CAMPAIGN_COUNT)
            .unwrap_or(0u32);

        count += 1;

        let campaign = Campaign {
            id: count,
            title,
            description,
            category,
            target_amount,
            current_amount: 0,
            recipient: recipient.clone(),
            status: CampaignStatus::Active,
            created_at: env.ledger().timestamp(),
        };

        let mut campaigns: Map<u32, Campaign> =
            env.storage().instance().get(&CAMPAIGNS).unwrap_or(Map::new(&env));

        campaigns.set(count, campaign.clone());
        env.storage().instance().set(&CAMPAIGNS, &campaigns);
        env.storage().instance().set(&CAMPAIGN_COUNT, &count);

        count
    }


    // ----------------------------------------------------
    // Mendapatkan total semua donasi dari semua campaign
    // ----------------------------------------------------
    pub fn get_total_donations(env: Env) -> i128 {
        let donations: Map<u32, Vec<Donation>> =
            env.storage().instance().get(&DONATIONS).unwrap_or(Map::new(&env));

        let mut total: i128 = 0;
        for (_id, list) in donations.iter() {
            for donation in list.iter() {
                total += donation.amount;
            }
        }

        total
    }

    // ----------------------------------------------------
    // Mendapatkan semua campaign
    // ----------------------------------------------------
    pub fn get_campaigns(env: Env) -> Vec<Campaign> {
        let campaigns: Map<u32, Campaign> =
            env.storage().instance().get(&CAMPAIGNS).unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);
        for (_id, camp) in campaigns.iter() {
            result.push_back(camp);
        }

        result
    }

    // ----------------------------------------------------
    // Mendapatkan campaign berdasarkan ID
    // ----------------------------------------------------
    pub fn get_campaign(env: Env, id: u32) -> Campaign {
    let campaigns: Map<u32, Campaign> =
        env.storage().instance().get(&CAMPAIGNS).unwrap_or(Map::new(&env));

    match campaigns.get(id) {
        Some(c) => c,
        None => panic!("Campaign not found"),
    }
}


    // ----------------------------------------------------
    // Donasi ke campaign tertentu
    // ----------------------------------------------------
    pub fn donate(env: Env, id: u32, donor: Address, amount: i128) {
        donor.require_auth();

        let mut campaigns: Map<u32, Campaign> =
            env.storage().instance().get(&CAMPAIGNS).unwrap_or(Map::new(&env));

        let mut campaign = campaigns.get(id).unwrap_or_else(|| panic!("Campaign not found"));
        if let CampaignStatus::Closed = campaign.status {
            panic!("Campaign already closed");
        }

        campaign.current_amount += amount;

        // Jika sudah mencapai target, ubah status ke Completed
        if campaign.current_amount >= campaign.target_amount {
            campaign.status = CampaignStatus::Completed;
        }

        campaigns.set(id, campaign.clone());
        env.storage().instance().set(&CAMPAIGNS, &campaigns);

        // Simpan data donasi
        let mut donations: Map<u32, Vec<Donation>> =
            env.storage().instance().get(&DONATIONS).unwrap_or(Map::new(&env));

        let mut list = donations.get(id).unwrap_or(Vec::new(&env));
        list.push_back(Donation {
            donor,
            amount,
            timestamp: env.ledger().timestamp(),
        });

        donations.set(id, list);
        env.storage().instance().set(&DONATIONS, &donations);
    }

    // ----------------------------------------------------
    // Mendapatkan semua donasi di campaign tertentu
    // ----------------------------------------------------
    pub fn get_donations(env: Env, id: u32) -> Vec<Donation> {
        let donations: Map<u32, Vec<Donation>> =
            env.storage().instance().get(&DONATIONS).unwrap_or(Map::new(&env));

        donations.get(id).unwrap_or(Vec::new(&env))
    }

    // ----------------------------------------------------
    // Menutup campaign (oleh admin atau penerima)
    // ----------------------------------------------------
    pub fn close_campaign(env: Env, id: u32, caller: Address) {
        caller.require_auth();

        let mut campaigns: Map<u32, Campaign> =
            env.storage().instance().get(&CAMPAIGNS).unwrap_or(Map::new(&env));

        let mut campaign = campaigns.get(id).unwrap_or_else(|| panic!("Campaign not found"));

        if caller != campaign.recipient {
            panic!("Only recipient can close the campaign");
        }

        campaign.status = CampaignStatus::Closed;

        campaigns.set(id, campaign);
        env.storage().instance().set(&CAMPAIGNS, &campaigns);
    }

    // ----------------------------------------------------
    // Menarik dana (withdraw)
    // ----------------------------------------------------
    pub fn withdraw(env: Env, id: u32, caller: Address) -> i128 {
        caller.require_auth();

        let mut campaigns: Map<u32, Campaign> =
            env.storage().instance().get(&CAMPAIGNS).unwrap_or(Map::new(&env));

        let mut campaign = campaigns.get(id).unwrap_or_else(|| panic!("Campaign not found"));

        if caller != campaign.recipient {
            panic!("Only recipient can withdraw");
        }

        match campaign.status {
            CampaignStatus::Completed | CampaignStatus::Closed => {
                let amount = campaign.current_amount;
                campaign.current_amount = 0;
                campaigns.set(id, campaign);
                env.storage().instance().set(&CAMPAIGNS, &campaigns);
                amount
            }
            _ => panic!("Campaign not completed or closed yet"),
        }
    }
}
