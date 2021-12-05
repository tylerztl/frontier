use bip39::{Language, Mnemonic, Seed};
use frontier_template_runtime::{
	AccountId, AuraConfig, BalancesConfig, EVMConfig, EthereumConfig, GenesisConfig, GrandpaConfig,
	Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use log::debug;
use sc_service::ChainType;
use sha3::{Digest, Keccak256};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{ecdsa, sr25519, Pair, Public, H160, H256, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{BlakeTwo256, Hash, IdentifyAccount, Verify};
use std::{collections::BTreeMap, convert::TryInto, str::FromStr};
use tiny_hderive::bip32::ExtendedPrivKey;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Helper function to derive `num_accounts` child pairs from mnemonics
/// Substrate derive function cannot be used because the derivation is different than Ethereum's
/// https://substrate.dev/rustdocs/v2.0.0/src/sp_core/ecdsa.rs.html#460-470
pub fn derive_bip44_pairs_from_mnemonic<TPublic: Public>(
	mnemonic: &str,
	num_accounts: u32,
) -> Vec<TPublic::Pair> {
	let seed = Mnemonic::from_phrase(mnemonic, Language::English)
		.map(|x| Seed::new(&x, ""))
		.expect("Wrong mnemonic provided");

	let mut childs = Vec::new();
	for i in 0..num_accounts {
		if let Some(child_pair) =
			ExtendedPrivKey::derive(seed.as_bytes(), format!("m/44'/60'/0'/0/{}", i).as_ref())
				.ok()
				.map(|account| TPublic::Pair::from_seed_slice(&account.secret()).ok())
				.flatten()
		{
			childs.push(child_pair);
		} else {
			log::error!("An error ocurred while deriving key {} from parent", i)
		}
	}
	childs
}

/// Helper function to get an AccountId from Key Pair
/// We need the full decompressed public key to derive an ethereum-style account
/// Substrate does not provide a method to obtain the full decompressed public key
/// Therefore, this function uses the secp256k1_ecdsa_recover method to recover the full key
/// A solution without using the private key would imply solving the secp256k1 curve equation
/// The latter is currently not possible with current substrate methods
pub fn get_account_id_from_pair<TPublic: Public>(pair: TPublic::Pair) -> Option<AccountId> {
	let test_message = [1u8; 32];
	let signature: [u8; 65] = pair.sign(&test_message).as_ref().try_into().ok()?;
	let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(
		&signature,
		BlakeTwo256::hash_of(&test_message).as_fixed_bytes(),
	)
	.ok()?;
	Some(H160::from(H256::from_slice(
		Keccak256::digest(&pubkey).as_slice(),
	)))
}

/// Function to generate accounts given a mnemonic and a number of child accounts to be generated
/// Defaults to a default mnemonic if no mnemonic is supplied
pub fn generate_accounts(mnemonic: String, num_accounts: u32) -> Vec<AccountId> {
	let childs = derive_bip44_pairs_from_mnemonic::<ecdsa::Public>(&mnemonic, num_accounts);
	debug!("Account Generation");
	childs
		.iter()
		.map(|par| {
			let account = get_account_id_from_pair::<ecdsa::Public>(par.clone());
			debug!(
				"private_key {} --------> Account {:x?}",
				sp_core::hexdisplay::HexDisplay::from(&par.clone().seed()),
				account
			);
			account
		})
		.flatten()
		.collect()
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	// Default mnemonic if none was provided
	let parent_mnemonic =
		"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string();
	// We prefund the standard dev accounts plus Gerald
	let mut accounts = generate_accounts(parent_mnemonic, 10);
	accounts.push(AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap());

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				accounts[0],
				// Pre-funded accounts
				accounts.clone(),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"USM\", \"SS58Prefix\": 1287, \"chainType\": \"ethereum\"}",
			)
			.expect("Provided valid json map"),
		),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	// Default mnemonic if none was provided
	let parent_mnemonic =
		"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string();
	// We prefund the standard dev accounts plus Gerald
	let mut accounts = generate_accounts(parent_mnemonic, 10);
	accounts.push(AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap());

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
				],
				// Sudo account
				accounts[0],
				// Pre-funded accounts
				accounts.clone(),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"UNIT\", \"SS58Prefix\": 1287}",
			)
			.expect("Provided valid json map"),
		),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 80.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 80))
				.collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities
				.iter()
				.map(|x| (x.1.clone(), 1))
				.collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					// H160 address of Alice dev account
					// Derived from SS58 (42 prefix) address
					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
						.expect("internal H160 is valid; qed"),
					pallet_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					// H160 address of CI test runner account
					H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
						.expect("internal H160 is valid; qed"),
					pallet_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
		},
		ethereum: EthereumConfig {},
		dynamic_fee: Default::default(),
	}
}
