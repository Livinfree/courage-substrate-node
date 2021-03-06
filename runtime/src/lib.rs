#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_std::prelude::*;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys, MultiSignature,
	transaction_validity::{TransactionValidity, TransactionSource}, ModuleId
};
use sp_runtime::traits::{
	BlakeTwo256, Block as BlockT, IdentityLookup, Verify, IdentifyAccount, NumberFor, Saturating,
};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_balances::Call as BalancesCall;
pub use sp_runtime::{Permill, Perbill, Percent};
pub use frame_support::{
	construct_runtime, parameter_types, StorageValue,
	traits::{KeyOwnerProofSystem, Randomness, LockIdentifier, EnsureOrigin},
	weights::{
		Weight, IdentityFee,
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
	},
};
use frame_system::{EnsureRoot, EnsureOneOf, EnsureNever, EnsureSigned};
use sp_runtime::transaction_validity::{ TransactionPriority };
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime where Call: From<C> {
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = Call;
}

//Additional code

use sp_runtime::MultiSigner;


use im_online::sr25519::AuthorityId as ImOnlineId;


use sp_runtime::traits::{ OpaqueKeys };
impl session::historical::Trait for Runtime {
	type FullIdentification = staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = staking::ExposureOf<Runtime>;
}


use sp_runtime::curve::PiecewiseLinear;
pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}


pub use contracts::Schedule as ContractsSchedule;


/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("node-template"),
	impl_name: create_runtime_str!("node-template"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	/// Assume 10% of weight for average on_initialize calls.
	pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
		.saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
	pub const Version: RuntimeVersion = VERSION;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Trait for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = ();
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = IdentityLookup<AccountId>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Maximum weight of each block.
	type MaximumBlockWeight = MaximumBlockWeight;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The weight of the overhead invoked on the block import process, independent of the
	/// extrinsics included in that block.
	type BlockExecutionWeight = BlockExecutionWeight;
	/// The base weight of any extrinsic processed by the runtime, independent of the
	/// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
	type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
	/// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
	/// idependent of the logic of that extrinsics. (Roughly max block weight - average on
	/// initialize cost).
	type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
	/// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
	type MaximumBlockLength = MaximumBlockLength;
	/// Portion of the block weight that is available to all normal transactions.
	type AvailableBlockRatio = AvailableBlockRatio;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
}

impl pallet_aura::Trait for Runtime {
	type AuthorityId = AuraId;
}

impl pallet_grandpa::Trait for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = ();

	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;

	type HandleEquivocation = ();

	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Trait for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Trait for Runtime {
	type Currency = Balances;
	type OnTransactionPayment = ();
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Trait for Runtime {
	type Event = Event;
	type Call = Call;
}

impl assets::Trait for Runtime { 
	type Event = Event;
	type Balance = Balance;
	type AssetId = u32;
}

parameter_types! { 
	pub const ProofLimit: u32 = 10;
}

impl atomic_swap::Trait for Runtime { 
	type Event = Event;
	type SwapAction = atomic_swap::BalanceSwapAction<AccountId, Balances>;
	type ProofLimit = ProofLimit;
}

parameter_types! { 
	pub const TombstoneDeposit: Balance = 16_000_000_000;
	pub const RentByteFee: Balance = 4_000_000_000;
	pub const RentDepositOffset: Balance = 1_000_000_000_000;
	pub const SurchargeReward: Balance = 150_000_000_000;
}

impl contracts::Trait for Runtime { 
	type Time = Timestamp;
	type Currency = Balances;
	type Randomness = RandomnessCollectiveFlip;
	type Event = Event;
	type DetermineContractAddress = contracts::SimpleAddressDeterminer<Runtime>;
	type TrieIdGenerator = contracts::TrieIdFromParentCounter<Runtime>;
	type RentPayment = ();
	type SignedClaimHandicap = contracts::DefaultSignedClaimHandicap;
	type TombstoneDeposit = TombstoneDeposit;
	type StorageSizeOffset = contracts::DefaultStorageSizeOffset;
	type RentByteFee = RentByteFee;
	type RentDepositOffset = RentDepositOffset;
	type SurchargeReward = SurchargeReward;
	type MaxDepth = contracts::DefaultMaxDepth;
	type MaxValueSize = contracts::DefaultMaxValueSize;
	type WeightPrice = pallet_transaction_payment::Module<Self>;
}

parameter_types! { 
	pub  MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl scheduler::Trait for Runtime { 
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = ();
}

parameter_types! { 
	pub const MaxProposals: u32 = 100;
	pub const MotionDuration: BlockNumber = 28_800;
	pub const CouncilMaxMembers: u32 = 100;
}

impl collective::Trait for Runtime { 
	type Event = Event;
	type WeightInfo = ();
	type Origin = Origin;
	type Proposal = Call;
	type MaxProposals = MaxProposals;
	type MotionDuration = MotionDuration;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = collective::PrimeDefaultVote;
}

parameter_types! { 
	pub const ElectionsPhragmenModuleId: LockIdentifier = *b"elnPrMdl";
	pub const CandidacyBondPhragmen: Balance = 5000;
	pub const VotingBondPhragmen: Balance = 500;
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
	pub const TermDuration: BlockNumber = 486_600;
}

impl elections_phragmen::Trait for Runtime { 
	type Event = Event;
	type ModuleId = ElectionsPhragmenModuleId;
	type Currency = Balances;
	type ChangeMembers = Collective;
	type InitializeMembers = Collective;
	type CurrencyToVote = ();
	type CandidacyBond = CandidacyBondPhragmen;
	type VotingBond = VotingBondPhragmen;
	type LoserCandidate = ();
	type BadReport = ();
	type KickedMember = ();
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = ();
}

parameter_types! { 
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"trsryMdl");
	pub const TipCountdown: BlockNumber = 28800;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 100_000_000_000_000;
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 100_000_000_000_000;
	pub const SpendPeriod: BlockNumber = 28800;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const DataDepositPerByte: Balance = 1_000_000_000_000;
	pub const BountyDepositBase: Balance = 100_000_000_000_000;
	pub const BountyDepositPayoutDelay: BlockNumber = 28800;
	pub const BountyUpdatePeriod: BlockNumber = 403200;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 500_000_000_000_000;
	pub const MaximumReasonLength: u32 = 16384;
}

impl treasury::Trait for Runtime { 
	type Event = Event;
	type ModuleId = TreasuryModuleId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type Tippers = ElectionsPhragmen;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type WeightInfo = ();
	type DataDepositPerByte = DataDepositPerByte;
	type OnSlash = ();
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type MaximumReasonLength = MaximumReasonLength;
}

parameter_types! { 
	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * 20;
	pub const LaunchPeriodDemocracy: BlockNumber = 28 * 24 * 60 * 20;
	pub const VotingPeriodDemocracy: BlockNumber = 28 * 24 * 60 * 20;
	pub const MinimumDeposit: Balance = 100 * 100_000_000_000_000;
	pub const InstantAllowed: bool = true;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * 24 * 60 * 20;
	pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * 20;
	pub const PreimageByteDeposit: Balance = 1_000_000_000_000;
	pub const MaxVotes: u32 = 100;
}

impl democracy::Trait for Runtime { 
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriodDemocracy;
	type VotingPeriod = VotingPeriodDemocracy;
	type MinimumDeposit = MinimumDeposit;
	type ExternalOrigin = EnsureRoot<AccountId>;
	type ExternalMajorityOrigin = EnsureSigned<AccountId>;
	type ExternalDefaultOrigin = EnsureSigned<AccountId>;
	type FastTrackOrigin = EnsureSigned<AccountId>;
	type InstantOrigin = EnsureSigned<AccountId>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type CancellationOrigin = EnsureNever<AccountId>;
	type VetoOrigin = EnsureNever<AccountId>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = EnsureSigned<AccountId>;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = ();
}

parameter_types! { 
	pub const ElectionsModuleId: LockIdentifier = *b"elcnsMdl";
	pub const CandidacyBond: Balance = 9;
	pub const VotingBond: Balance = 3;
	pub const VotingFee: Balance = 1;
	pub const MinimumVotingLock: Balance = 1;
	pub const PresentSlashPerVoter: Balance = 1;
	pub const CarryCount: u32 = 2;
	pub const InactiveGracePeriod: u32 = 1;
	pub const VotingPeriod: BlockNumber = 1000;
	pub const DecayRatio: u32 = 24;
}

impl elections::Trait for Runtime { 
	type Event = Event;
	type ModuleId = ElectionsModuleId;
	type Currency = Balances;
	type BadPresentation = ();
	type BadReaper = ();
	type BadVoterIndex = ();
	type LoserCandidate = ();
	type ChangeMembers = ();
	type CandidacyBond = CandidacyBond;
	type VotingBond = VotingBond;
	type VotingFee = VotingFee;
	type MinimumVotingLock = MinimumVotingLock;
	type PresentSlashPerVoter = PresentSlashPerVoter;
	type CarryCount = CarryCount;
	type InactiveGracePeriod = InactiveGracePeriod;
	type VotingPeriod = VotingPeriod;
	type DecayRatio = DecayRatio;
}

parameter_types! { 
	pub const BasicDeposit: Balance = 1_000_000_000_000_000;
	pub const FieldDeposit: Balance = 250_000_000_000_000;
	pub const SubAccountDeposit: Balance = 200_000_000_000_000;
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl identity::Trait for Runtime { 
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = ();
	type ForceOrigin = EnsureRoot<AccountId>;
	type RegistrarOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

parameter_types! { 
	pub const EpochDuration: u64 = 200;
	pub const ExpectedBlockTime: u64 = 3000;
}

impl babe::Trait for Runtime { 
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = babe::ExternalTrigger;
	type KeyOwnerProofSystem = ();
	type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		babe::AuthorityId,
	)>>::Proof;
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		babe::AuthorityId,
	)>>::IdentificationTuple;
	type HandleEquivocation = ();
	type WeightInfo = ();
}

parameter_types! { 
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: staking::EraIndex = 24 * 28;
	pub const SlashDeferDuration: staking::EraIndex = 24 * 7;
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const ElectionLookahead: BlockNumber = 50;
	pub const MaxIterations: u32 = 10;
	pub  MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
	pub const UnsignedPriorityStaking: TransactionPriority = TransactionPriority::max_value() / 2;
}

impl staking::Trait for Runtime { 
	type Event = Event;
	type Currency = Balances;
	type Call = Call;
	type UnixTime = Timestamp;
	type CurrencyToVote = ();
	type RewardRemainder = ();
	type Slash = ();
	type Reward = ();
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	type SlashCancelOrigin = EnsureOneOf<AccountId, EnsureRoot<AccountId>, EnsureRoot<AccountId>>;
	type SessionInterface = Self;
	type RewardCurve = RewardCurve;
	type NextNewSession = Session;
	type ElectionLookahead = ElectionLookahead;
	type MaxIterations = MaxIterations;
	type MinSolutionScoreBump = MinSolutionScoreBump;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type UnsignedPriority = UnsignedPriorityStaking;
	type WeightInfo = ();
}

parameter_types! { 
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl session::Trait for Runtime { 
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ();
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = ();
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

parameter_types! { 
	pub const UncleGenerations: BlockNumber = 5;
}

impl authorship::Trait for Runtime { 
	type EventHandler = ();
	type FilterUncle = ();
	type FindAuthor = session::FindAccountFromAuthorIndex<Self, Babe>;
	type UncleGenerations = UncleGenerations;
}

impl authority_discovery::Trait for Runtime { 
}

parameter_types! { 
	pub const SessionDuration: BlockNumber = 200;
	pub const UnsignedPriorityImOnline: TransactionPriority = TransactionPriority::max_value();
}

impl im_online::Trait for Runtime { 
	type Event = Event;
	type AuthorityId = ImOnlineId;
	type WeightInfo = ();
	type ReportUnresponsiveness = ();
	type SessionDuration = SessionDuration;
	type UnsignedPriority = UnsignedPriorityImOnline;
}

impl membership::Trait for Runtime { 
	type Event = Event;
	type AddOrigin = EnsureRoot<AccountId>;
	type RemoveOrigin = EnsureRoot<AccountId>;
	type SwapOrigin = EnsureRoot<AccountId>;
	type ResetOrigin = EnsureRoot<AccountId>;
	type PrimeOrigin = EnsureRoot<AccountId>;
	type MembershipChanged = ();
	type MembershipInitialized = ();
}

parameter_types! { 
	pub const DepositBase: Balance = 543_000_000_000_000;
	pub const DepositFactor: Balance = 192_000_000_000_000;
	pub const MaxSignatories: u16 = 100;
}

impl multisig::Trait for Runtime { 
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}

parameter_types! { 
	pub const MinLength: usize = 8;
	pub const MaxLength: usize = 32;
	pub const ReservationFee: u128 = 100;
}

impl nicks::Trait for Runtime { 
	type Currency = Balances;
	type Event = Event;
	type ForceOrigin = EnsureRoot<AccountId>;
	type MinLength = MinLength;
	type MaxLength = MaxLength;
	type Slashed = ();
	type ReservationFee = ReservationFee;
}

impl did::Trait for Runtime { 
	type Event = Event;
	type Public = MultiSigner;
	type Signature = Signature;
}

impl registrar::Trait for Runtime { 
	type Event = Event;
}

impl product_registry::Trait for Runtime { 
	type Event = Event;
	type CreateRoleOrigin = registrar::EnsureOrg<Runtime>;
}

impl product_tracking::Trait for Runtime { 
	type Event = Event;
	type CreateRoleOrigin = registrar::EnsureOrg<Runtime>;
}

parameter_types! { 
	pub const CandidateDepositScoredPool: Balance = 25;
	pub const PeriodScoredPool: BlockNumber = 4;
}

impl scored_pool::Trait for Runtime { 
	type Currency = Balances;
	type Score = u64;
	type Event = Event;
	type CandidateDeposit = CandidateDepositScoredPool;
	type Period = PeriodScoredPool;
	type MembershipInitialized = ();
	type MembershipChanged = ();
	type ScoreOrigin = EnsureRoot<AccountId>;
	type KickOrigin = EnsureRoot<AccountId>;
}

parameter_types! { 
	pub const SocietyModuleId: ModuleId = ModuleId(*b"soctyMdl");
	pub const CandidateDeposit: Balance = 1000_000_000_000_000;
	pub const WrongSideDeduction: Balance = 200_000_000_000_000;
	pub const MaxStrikes: u32 = 10;
	pub const PeriodSpend: Balance = 50_000_000_000_000_000;
	pub const RotationPeriod: BlockNumber = 80 * 1200;
	pub const MaxLockDuration: BlockNumber = 36 * 30 * 28800;
	pub const ChallengePeriod: BlockNumber = 7 * 28800;
}

impl society::Trait for Runtime { 
	type Event = Event;
	type ModuleId = SocietyModuleId;
	type Currency = Balances;
	type Randomness = RandomnessCollectiveFlip;
	type CandidateDeposit = CandidateDeposit;
	type WrongSideDeduction = WrongSideDeduction;
	type MaxStrikes = MaxStrikes;
	type PeriodSpend = PeriodSpend;
	type MembershipChanged = ();
	type RotationPeriod = RotationPeriod;
	type MaxLockDuration = MaxLockDuration;
	type FounderSetOrigin = EnsureRoot<AccountId>;
	type SuspensionJudgementOrigin = society::EnsureFounder<Runtime>;
	type ChallengePeriod = ChallengePeriod;
}

impl utility::Trait for Runtime { 
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! { 
	pub const MinVestedTransfer: Balance = 1000;
}

impl vesting::Trait for Runtime { 
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ();
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
}

impl rbac::Trait for Runtime { 
	type Event = Event;
	type CreateRoleOrigin = registrar::EnsureOrg<Runtime>;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Aura: pallet_aura::{Module, Config<T>, Inherent},
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Module, Storage},
		Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},		Assets: assets::{Module, Call, Storage, Event<T> },
		AtomicSwap: atomic_swap::{Module, Call, Storage, Event<T> },
		Contracts: contracts::{Module, Call, Config, Storage, Event<T> },
		Scheduler: scheduler::{Module, Call, Storage, Event<T> },
		Collective: collective::{Module, Call, Storage, Event<T>, Origin<T>, Config<T> },
		ElectionsPhragmen: elections_phragmen::{Module, Call, Storage, Event<T>, Config<T> },
		Treasury: treasury::{Module, Call, Storage, Event<T> },
		Democracy: democracy::{Module, Call, Storage, Event<T> },
		Elections: elections::{Module, Call, Storage, Config<T>, Event<T> },
		Identity: identity::{Module, Call, Storage, Event<T> },
		Babe: babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned },
		Staking: staking::{Module, Call, Config<T>, Storage, Event<T>, ValidateUnsigned },
		Session: session::{Module, Call, Storage, Event, Config<T> },
		Authorship: authorship::{Module, Call, Storage, Inherent },
		AuthorityDiscovery: authority_discovery::{Module, Call, Storage },
		ImOnline: im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T> },
		Membership: membership::{Module, Call, Storage, Event<T>, Config<T> },
		Multisig: multisig::{Module, Call, Storage, Event<T> },
		Nicks: nicks::{Module, Call, Event<T>, Storage },
		Did: did::{Module, Call, Storage, Event<T> },
		Registrar: registrar::{Module, Call, Storage, Event<T> },
		ProductRegistry: product_registry::{Module, Call, Storage, Event<T> },
		ProductTracking: product_tracking::{Module, Call, Storage, Event<T> },
		ScoredPool: scored_pool::{Module, Call, Storage, Event<T>, Config<T> },
		Society: society::{Module, Call, Storage, Event<T>, Config<T> },
		Utility: utility::{Module, Call, Event, Storage },
		Vesting: vesting::{Module, Call, Storage, Event<T> },
		Rbac: rbac::{Module, Call, Storage, Event<T>, Config<T> },

	}
);

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllModules,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> u64 {
			Aura::slot_duration()
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Module as SystemBench;
			impl frame_system_benchmarking::Trait for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}
