use crate as sugarfunge_bridge;
use frame_support::{assert_ok, parameter_types, traits::Everything, PalletId};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};
use sugarfunge_primitives::Balance;

pub const MILLICENTS: Balance = 10_000_000_000_000;

parameter_types! {
    pub const CreateAssetClassDeposit: Balance = 500 * MILLICENTS;
    pub const CreateCurrencyClassDeposit: Balance = 500 * MILLICENTS;
    pub const CreateBagDeposit: Balance = 1;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxClassMetadata: u32 = 1;
    pub const MaxAssetMetadata: u32 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
}

impl sugarfunge_asset::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CreateAssetClassDeposit = CreateAssetClassDeposit;
    type Currency = Balances;
    type AssetId = u64;
    type ClassId = u64;
    type MaxClassMetadata = MaxClassMetadata;
    type MaxAssetMetadata = MaxAssetMetadata;
}

parameter_types! {
    pub const MaxAssets: u32 = 20;
}

parameter_types! {
    pub const TestChainId: u8 = 5;
    pub const ProposalLifetime: u64 = 50;
    pub const BridgeModuleId: PalletId = PalletId(*b"sug/brdg");
    pub const MaxResourceMetadata: u32 = 128;
    pub const DefaultRelayerThreshold: u32 = 1;
    pub const MaxVotes: u32 = 100;
}

impl sugarfunge_bridge::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = BridgeModuleId;
    type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type Proposal = RuntimeCall;
    type ChainId = TestChainId;
    type ProposalLifetime = ProposalLifetime;
    type MaxResourceMetadata = MaxResourceMetadata;
    type DefaultRelayerThreshold = DefaultRelayerThreshold;
    type MaxVotes = MaxVotes;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Asset: sugarfunge_asset::{Pallet, Call, Storage, Event<T>},
        Bridge: sugarfunge_bridge::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// pub const BRIDGE_ID: u64 =
pub const RELAYER_A: u64 = 0x2;
pub const RELAYER_B: u64 = 0x3;
pub const RELAYER_C: u64 = 0x4;
pub const ENDOWED_BALANCE: u128 = 100_000_000;
pub const TEST_THRESHOLD: u32 = 2;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let bridge_id = <Test as sugarfunge_bridge::Config>::PalletId::get().into_account_truncating();
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(bridge_id, ENDOWED_BALANCE)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn new_test_ext_initialized(
    src_id: sugarfunge_bridge::ChainId,
    r_id: sugarfunge_bridge::ResourceId,
    resource: Vec<u8>,
) -> sp_io::TestExternalities {
    let mut t = new_test_ext();
    t.execute_with(|| {
        // Set and check threshold
        assert_ok!(Bridge::set_threshold(RuntimeOrigin::root(), TEST_THRESHOLD));
        assert_eq!(Bridge::relayer_threshold(), TEST_THRESHOLD);
        // Add relayers
        assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), RELAYER_A));
        assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), RELAYER_B));
        assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), RELAYER_C));
        // Whitelist chain
        assert_ok!(Bridge::whitelist_chain(RuntimeOrigin::root(), src_id));
        // Set and check resource ID mapped to some junk data
        assert_ok!(Bridge::set_resource(
            RuntimeOrigin::root(),
            r_id,
            resource.try_into().unwrap()
        ));
        assert_eq!(Bridge::resource_exists(r_id), true);
    });
    t
}

// Checks events against the latest. A contiguous set of events must be provided. They must
// include the most recent event, but do not have to include every past event.
pub fn assert_events(mut expected: Vec<RuntimeEvent>) {
    let mut actual: Vec<RuntimeEvent> = system::Pallet::<Test>::events()
        .iter()
        .map(|e| e.event.clone())
        .collect();

    expected.reverse();

    for evt in expected {
        let next = actual.pop().expect("event expected");
        assert_eq!(next, evt.into(), "Events don't match (actual,expected)");
    }
}
