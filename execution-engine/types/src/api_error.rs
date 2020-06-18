//! Contains [`ApiError`] and associated helper functions.

use core::{
    fmt::{self, Debug, Formatter},
    u16, u8,
};

use crate::{
    account::{
        AddKeyFailure, RemoveKeyFailure, SetThresholdFailure, TryFromIntError,
        TryFromSliceForPublicKeyError, UpdateKeyFailure,
    },
    encoding,
    system_contract_errors::{mint, pos},
    CLValueError,
};

/// All `Error` variants defined in this library other than `Error::User` will convert to a `u32`
/// value less than or equal to `RESERVED_ERROR_MAX`.
const RESERVED_ERROR_MAX: u32 = u16::MAX as u32; // 0..=65535

/// Proof of Stake errors (defined in "contracts/system/pos/src/error.rs") will have this value
/// added to them when being converted to a `u32`.
const POS_ERROR_OFFSET: u32 = RESERVED_ERROR_MAX - u8::MAX as u32; // 65280..=65535

/// Mint errors (defined in "contracts/system/mint/src/error.rs") will have this value
/// added to them when being converted to a `u32`.
const MINT_ERROR_OFFSET: u32 = (POS_ERROR_OFFSET - 1) - u8::MAX as u32; // 65024..=65279

/// Minimum value of user error's inclusive range.
const USER_ERROR_MIN: u32 = RESERVED_ERROR_MAX + 1;

/// Maximum value of user error's inclusive range.
const USER_ERROR_MAX: u32 = 2 * RESERVED_ERROR_MAX + 1;

/// Minimum value of Mint error's inclusive range.
const MINT_ERROR_MIN: u32 = MINT_ERROR_OFFSET;

/// Maximum value of Mint error's inclusive range.
const MINT_ERROR_MAX: u32 = POS_ERROR_OFFSET - 1;

/// Minimum value of Proof of Stake error's inclusive range.
const POS_ERROR_MIN: u32 = POS_ERROR_OFFSET;

/// Maximum value of Proof of Stake error's inclusive range.
const POS_ERROR_MAX: u32 = RESERVED_ERROR_MAX;

/// Errors which can be encountered while running a smart contract.
///
/// An `ApiError` can be converted to a `u32` in order to be passed via the execution engine's
/// `ext_ffi::revert()` function.  This means the information each variant can convey is limited.
///
/// The variants are split into numeric ranges as follows:
///
/// | Inclusive range | Variant(s)                                   |
/// | ----------------| ---------------------------------------------|
/// | [1, 65023]      | all except `Mint`, `ProofOfStake` and `User` |
/// | [65024, 65279]  | `Mint`                                       |
/// | [65280, 65535]  | `ProofOfStake`                               |
/// | [65536, 131071] | `User`                                       |
///
/// ## Mappings
///
/// The expanded mapping of all variants to their numerical equivalents is as follows:
/// ```
/// # use casperlabs_types::ApiError::{self, *};
/// # macro_rules! show_and_check {
/// #     ($lhs:literal => $rhs:expr) => {
/// #         assert_eq!($lhs as u32, ApiError::from($rhs).into());
/// #     };
/// # }
/// // General system errors:
/// # show_and_check!(
/// 1 => None
/// # );
/// # show_and_check!(
/// 2 => MissingArgument
/// # );
/// # show_and_check!(
/// 3 => InvalidArgument
/// # );
/// # show_and_check!(
/// 4 => Deserialize
/// # );
/// # show_and_check!(
/// 5 => Read
/// # );
/// # show_and_check!(
/// 6 => ValueNotFound
/// # );
/// # show_and_check!(
/// 7 => ContractNotFound
/// # );
/// # show_and_check!(
/// 8 => GetKey
/// # );
/// # show_and_check!(
/// 9 => UnexpectedKeyVariant
/// # );
/// # show_and_check!(
/// 10 => UnexpectedContractRefVariant
/// # );
/// # show_and_check!(
/// 11 => InvalidPurseName
/// # );
/// # show_and_check!(
/// 12 => InvalidPurse
/// # );
/// # show_and_check!(
/// 13 => UpgradeContractAtURef
/// # );
/// # show_and_check!(
/// 14 => Transfer
/// # );
/// # show_and_check!(
/// 15 => NoAccessRights
/// # );
/// # show_and_check!(
/// 16 => CLTypeMismatch
/// # );
/// # show_and_check!(
/// 17 => ApiError::EncodingExcessiveDiscriminants
/// # );
/// # show_and_check!(
/// 18 => ApiError::EncodingEndOfSlice
/// # );
/// # show_and_check!(
/// 19 => ApiError::EncodingLeftOverBytes
/// # );
/// # show_and_check!(
/// 20 => ApiError::EncodingInvalidUtf8
/// # );
/// # show_and_check!(
/// 21 => ApiError::EncodingInvalidBool
/// # );
/// # show_and_check!(
/// 22 => ApiError::EncodingInvalidChar
/// # );
/// # show_and_check!(
/// 23 => ApiError::EncodingInvalidTag
/// # );
/// # show_and_check!(
/// 24 => ApiError::EncodingUnsupported
/// # );
/// # show_and_check!(
/// 25 => ApiError::EncodingSizeLimit
/// # );
/// # show_and_check!(
/// 26 => ApiError::EncodingSequenceMustHaveLength
/// # );
/// # show_and_check!(
/// 27 => ApiError::EncodingCustom
/// # );
/// # show_and_check!(
/// 28 => ApiError::MaxKeysLimit
/// # );
/// # show_and_check!(
/// 29 => ApiError::DuplicateKey
/// # );
/// # show_and_check!(
/// 30 => ApiError::PermissionDenied
/// # );
/// # show_and_check!(
/// 31 => ApiError::MissingKey
/// # );
/// # show_and_check!(
/// 32 => ApiError::ThresholdViolation
/// # );
/// # show_and_check!(
/// 33 => ApiError::KeyManagementThreshold
/// # );
/// # show_and_check!(
/// 34 => ApiError::DeploymentThreshold
/// # );
/// # show_and_check!(
/// 35 => ApiError::InsufficientTotalWeight
/// # );
/// # show_and_check!(
/// 36 => ApiError::InvalidSystemContract
/// # );
/// # show_and_check!(
/// 37 => ApiError::PurseNotCreated
/// # );
/// # show_and_check!(
/// 38 => ApiError::Unhandled
/// # );
/// # show_and_check!(
/// 39 => ApiError::BufferTooSmall
/// # );
/// # show_and_check!(
/// 40 => ApiError::HostBufferEmpty
/// # );
/// # show_and_check!(
/// 41 => ApiError::HostBufferFull
/// # );
/// # show_and_check!(
/// 42 => ApiError::AllocLayout
/// # );
///
/// // Mint errors:
/// use casperlabs_types::system_contract_errors::mint::Error as MintError;
/// # show_and_check!(
/// 65_024 => MintError::InsufficientFunds
/// # );
/// # show_and_check!(
/// 65_025 => MintError::SourceNotFound
/// # );
/// # show_and_check!(
/// 65_026 => MintError::DestNotFound
/// # );
/// # show_and_check!(
/// 65_027 => MintError::InvalidURef
/// # );
/// # show_and_check!(
/// 65_028 => MintError::InvalidAccessRights
/// # );
/// # show_and_check!(
/// 65_029 => MintError::InvalidNonEmptyPurseCreation
/// # );
/// # show_and_check!(
/// 65_030 => MintError::Storage
/// # );
/// # show_and_check!(
/// 65_031 => MintError::PurseNotFound
/// # );
///
/// // Proof of stake errors:
/// use casperlabs_types::system_contract_errors::pos::Error as PosError;
/// # show_and_check!(
/// 65_280 => PosError::NotBonded
/// # );
/// # show_and_check!(
/// 65_281 => PosError::TooManyEventsInQueue
/// # );
/// # show_and_check!(
/// 65_282 => PosError::CannotUnbondLastValidator
/// # );
/// # show_and_check!(
/// 65_283 => PosError::SpreadTooHigh
/// # );
/// # show_and_check!(
/// 65_284 => PosError::MultipleRequests
/// # );
/// # show_and_check!(
/// 65_285 => PosError::BondTooSmall
/// # );
/// # show_and_check!(
/// 65_286 => PosError::BondTooLarge
/// # );
/// # show_and_check!(
/// 65_287 => PosError::UnbondTooLarge
/// # );
/// # show_and_check!(
/// 65_288 => PosError::BondTransferFailed
/// # );
/// # show_and_check!(
/// 65_289 => PosError::UnbondTransferFailed
/// # );
/// # show_and_check!(
/// 65_290 => PosError::TimeWentBackwards
/// # );
/// # show_and_check!(
/// 65_291 => PosError::StakesNotFound
/// # );
/// # show_and_check!(
/// 65_292 => PosError::PaymentPurseNotFound
/// # );
/// # show_and_check!(
/// 65_293 => PosError::PaymentPurseKeyUnexpectedType
/// # );
/// # show_and_check!(
/// 65_294 => PosError::PaymentPurseBalanceNotFound
/// # );
/// # show_and_check!(
/// 65_295 => PosError::BondingPurseNotFound
/// # );
/// # show_and_check!(
/// 65_296 => PosError::BondingPurseKeyUnexpectedType
/// # );
/// # show_and_check!(
/// 65_297 => PosError::RefundPurseKeyUnexpectedType
/// # );
/// # show_and_check!(
/// 65_298 => PosError::RewardsPurseNotFound
/// # );
/// # show_and_check!(
/// 65_299 => PosError::RewardsPurseKeyUnexpectedType
/// # );
/// # show_and_check!(
/// 65_300 => PosError::StakesKeyDeserializationFailed
/// # );
/// # show_and_check!(
/// 65_301 => PosError::StakesDeserializationFailed
/// # );
/// # show_and_check!(
/// 65_302 => PosError::SystemFunctionCalledByUserAccount
/// # );
/// # show_and_check!(
/// 65_303 => PosError::InsufficientPaymentForAmountSpent
/// # );
/// # show_and_check!(
/// 65_304 => PosError::FailedTransferToRewardsPurse
/// # );
/// # show_and_check!(
/// 65_305 => PosError::FailedTransferToAccountPurse
/// # );
/// # show_and_check!(
/// 65_306 => PosError::SetRefundPurseCalledOutsidePayment
/// # );
///
/// // User-defined errors:
/// # show_and_check!(
/// 65_536 => User(0)
/// # );
/// # show_and_check!(
/// 65_537 => User(1)
/// # );
/// # show_and_check!(
/// 65_538 => User(2)
/// # );
/// # show_and_check!(
/// 131_071 => User(u16::max_value())
/// # );
/// ```
///
/// Users can specify a C-style enum and implement `From` to ease usage of
/// `casperlabs_contract::runtime::revert()`, e.g.
/// ```
/// use casperlabs_types::ApiError;
///
/// #[repr(u16)]
/// enum FailureCode {
///     Zero = 0,  // 65,536 as an ApiError::User
///     One,       // 65,537 as an ApiError::User
///     Two        // 65,538 as an ApiError::User
/// }
///
/// impl From<FailureCode> for ApiError {
///     fn from(code: FailureCode) -> Self {
///         ApiError::User(code as u16)
///     }
/// }
///
/// assert_eq!(ApiError::User(1), FailureCode::One.into());
/// assert_eq!(65_536, u32::from(ApiError::from(FailureCode::Zero)));
/// assert_eq!(65_538, u32::from(ApiError::from(FailureCode::Two)));
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// Optional data was unexpectedly `None`.
    None,
    /// Specified argument not provided.
    MissingArgument,
    /// Argument not of correct type.
    InvalidArgument,
    /// Failed to deserialize a value.
    Deserialize,
    /// `casperlabs_contract::storage::read()` returned an error.
    Read,
    /// The given key returned a `None` value.
    ValueNotFound,
    /// Failed to find a specified contract.
    ContractNotFound,
    /// A call to `casperlabs_contract::runtime::get_key()` returned a failure.
    GetKey,
    /// The [`Key`](crate::Key) variant was not as expected.
    UnexpectedKeyVariant,
    /// The [`ContractRef`](crate::ContractRef) variant was not as expected.
    UnexpectedContractRefVariant,
    /// Invalid purse name given.
    InvalidPurseName,
    /// Invalid purse retrieved.
    InvalidPurse,
    /// Failed to upgrade contract at [`URef`](crate::URef).
    UpgradeContractAtURef,
    /// Failed to transfer motes.
    Transfer,
    /// The given [`URef`](crate::URef) has no access rights.
    NoAccessRights,
    /// A given type could not be constructed from a [`CLValue`](crate::CLValue).
    CLTypeMismatch,
    /// Returned if trying to serialize a type with more than 255 discriminants (e.g. a struct with
    /// too many fields, an enum with too many variants).
    EncodingExcessiveDiscriminants,
    /// Returned if input slice to deserializer is too short.
    EncodingEndOfSlice,
    /// Returned if input slice has remaining bytes after deserialization complete.
    EncodingLeftOverBytes,
    /// Returned if the deserializer attempts to deserialize a string that is not valid UTF-8.
    EncodingInvalidUtf8,
    /// Returned if the deserializer attempts to deserialize a bool that was not encoded as either
    /// `1` or `0`.
    EncodingInvalidBool,
    /// Returned if the deserializer attempts to deserialize a char that is not in the correct
    /// format.
    EncodingInvalidChar,
    /// Returned if the deserializer attempts to deserialize the tag of an enum that is not in the
    /// expected ranges.
    EncodingInvalidTag,
    /// Type to be serialized (e.g. `f32`) or serde method (e.g. `deserialize_any`) is unsupported.
    EncodingUnsupported,
    /// If (de)serializing a message takes more than the provided size limit, this error is
    /// returned.
    EncodingSizeLimit,
    /// Sequences of unknown length (like iterators) cannot be encoded.
    EncodingSequenceMustHaveLength,
    /// A custom error from Serde.
    EncodingCustom,
    /// There are already [`MAX_ASSOCIATED_KEYS`](crate::account::MAX_ASSOCIATED_KEYS)
    /// [`PublicKey`](crate::account::PublicKey)s associated with the given account.
    MaxKeysLimit,
    /// The given [`PublicKey`](crate::account::PublicKey) is already associated with the given
    /// account.
    DuplicateKey,
    /// Caller doesn't have sufficient permissions to perform the given action.
    PermissionDenied,
    /// The given [`PublicKey`](crate::account::PublicKey) is not associated with the given
    /// account.
    MissingKey,
    /// Removing/updating the given associated [`PublicKey`](crate::account::PublicKey) would cause
    /// the total [`Weight`](crate::account::Weight) of all remaining `PublicKey`s to fall below
    /// one of the action thresholds for the given account.
    ThresholdViolation,
    /// Setting the key-management threshold to a value lower than the deployment threshold is
    /// disallowed.
    KeyManagementThreshold,
    /// Setting the deployment threshold to a value greater than any other threshold is disallowed.
    DeploymentThreshold,
    /// Setting a threshold to a value greater than the total weight of associated keys is
    /// disallowed.
    InsufficientTotalWeight,
    /// The given `u32` doesn't map to a [`SystemContractType`](crate::SystemContractType).
    InvalidSystemContract,
    /// Failed to create a new purse.
    PurseNotCreated,
    /// An unhandled value, likely representing a bug in the code.
    Unhandled,
    /// The provided buffer is too small to complete an operation.
    BufferTooSmall,
    /// No data available in the host buffer.
    HostBufferEmpty,
    /// The host buffer has been set to a value and should be consumed first by a read operation.
    HostBufferFull,
    /// Could not lay out an array in memory
    AllocLayout,
    /// Error specific to Mint contract.
    Mint(u8),
    /// Error specific to Proof of Stake contract.
    ProofOfStake(u8),
    /// User-specified error code.  The internal `u16` value is added to `u16::MAX as u32 + 1` when
    /// an `Error::User` is converted to a `u32`.
    User(u16),
}

impl From<encoding::Error> for ApiError {
    fn from(error: encoding::Error) -> Self {
        match error {
            encoding::Error::ExcessiveDiscriminants => ApiError::EncodingExcessiveDiscriminants,
            encoding::Error::EndOfSlice => ApiError::EncodingEndOfSlice,
            encoding::Error::LeftOverBytes(_) => ApiError::EncodingLeftOverBytes,
            encoding::Error::InvalidUtf8Encoding(_) => ApiError::EncodingInvalidUtf8,
            encoding::Error::InvalidBoolEncoding(_) => ApiError::EncodingInvalidBool,
            encoding::Error::InvalidCharEncoding => ApiError::EncodingInvalidChar,
            encoding::Error::InvalidTagEncoding(_) => ApiError::EncodingInvalidTag,
            encoding::Error::Unsupported => ApiError::EncodingUnsupported,
            encoding::Error::SizeLimit => ApiError::EncodingSizeLimit,
            encoding::Error::SequenceMustHaveLength => ApiError::EncodingSequenceMustHaveLength,
            encoding::Error::Custom(_) => ApiError::EncodingCustom,
        }
    }
}

impl From<AddKeyFailure> for ApiError {
    fn from(error: AddKeyFailure) -> Self {
        match error {
            AddKeyFailure::MaxKeysLimit => ApiError::MaxKeysLimit,
            AddKeyFailure::DuplicateKey => ApiError::DuplicateKey,
            AddKeyFailure::PermissionDenied => ApiError::PermissionDenied,
        }
    }
}

impl From<UpdateKeyFailure> for ApiError {
    fn from(error: UpdateKeyFailure) -> Self {
        match error {
            UpdateKeyFailure::MissingKey => ApiError::MissingKey,
            UpdateKeyFailure::PermissionDenied => ApiError::PermissionDenied,
            UpdateKeyFailure::ThresholdViolation => ApiError::ThresholdViolation,
        }
    }
}

impl From<RemoveKeyFailure> for ApiError {
    fn from(error: RemoveKeyFailure) -> Self {
        match error {
            RemoveKeyFailure::MissingKey => ApiError::MissingKey,
            RemoveKeyFailure::PermissionDenied => ApiError::PermissionDenied,
            RemoveKeyFailure::ThresholdViolation => ApiError::ThresholdViolation,
        }
    }
}

impl From<SetThresholdFailure> for ApiError {
    fn from(error: SetThresholdFailure) -> Self {
        match error {
            SetThresholdFailure::KeyManagementThreshold => ApiError::KeyManagementThreshold,
            SetThresholdFailure::DeploymentThreshold => ApiError::DeploymentThreshold,
            SetThresholdFailure::PermissionDeniedError => ApiError::PermissionDenied,
            SetThresholdFailure::InsufficientTotalWeight => ApiError::InsufficientTotalWeight,
        }
    }
}

impl From<CLValueError> for ApiError {
    fn from(error: CLValueError) -> Self {
        match error {
            CLValueError::Serialization(bytesrepr_error) => bytesrepr_error.into(),
            CLValueError::Type(_) => ApiError::CLTypeMismatch,
        }
    }
}

// This conversion is not intended to be used by third party crates.
#[doc(hidden)]
impl From<TryFromIntError> for ApiError {
    fn from(_error: TryFromIntError) -> Self {
        ApiError::Unhandled
    }
}

impl From<TryFromSliceForPublicKeyError> for ApiError {
    fn from(_error: TryFromSliceForPublicKeyError) -> Self {
        ApiError::Deserialize
    }
}

impl From<mint::Error> for ApiError {
    fn from(error: mint::Error) -> Self {
        ApiError::Mint(error as u8)
    }
}

impl From<pos::Error> for ApiError {
    fn from(error: pos::Error) -> Self {
        ApiError::ProofOfStake(error as u8)
    }
}

impl From<ApiError> for u32 {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::None => 1,
            ApiError::MissingArgument => 2,
            ApiError::InvalidArgument => 3,
            ApiError::Deserialize => 4,
            ApiError::Read => 5,
            ApiError::ValueNotFound => 6,
            ApiError::ContractNotFound => 7,
            ApiError::GetKey => 8,
            ApiError::UnexpectedKeyVariant => 9,
            ApiError::UnexpectedContractRefVariant => 10,
            ApiError::InvalidPurseName => 11,
            ApiError::InvalidPurse => 12,
            ApiError::UpgradeContractAtURef => 13,
            ApiError::Transfer => 14,
            ApiError::NoAccessRights => 15,
            ApiError::CLTypeMismatch => 16,
            ApiError::EncodingExcessiveDiscriminants => 17,
            ApiError::EncodingEndOfSlice => 18,
            ApiError::EncodingLeftOverBytes => 19,
            ApiError::EncodingInvalidUtf8 => 20,
            ApiError::EncodingInvalidBool => 21,
            ApiError::EncodingInvalidChar => 22,
            ApiError::EncodingInvalidTag => 23,
            ApiError::EncodingUnsupported => 24,
            ApiError::EncodingSizeLimit => 25,
            ApiError::EncodingSequenceMustHaveLength => 26,
            ApiError::EncodingCustom => 27,
            ApiError::MaxKeysLimit => 28,
            ApiError::DuplicateKey => 29,
            ApiError::PermissionDenied => 30,
            ApiError::MissingKey => 31,
            ApiError::ThresholdViolation => 32,
            ApiError::KeyManagementThreshold => 33,
            ApiError::DeploymentThreshold => 34,
            ApiError::InsufficientTotalWeight => 35,
            ApiError::InvalidSystemContract => 36,
            ApiError::PurseNotCreated => 37,
            ApiError::Unhandled => 38,
            ApiError::BufferTooSmall => 39,
            ApiError::HostBufferEmpty => 40,
            ApiError::HostBufferFull => 41,
            ApiError::AllocLayout => 42,
            ApiError::Mint(value) => MINT_ERROR_OFFSET + u32::from(value),
            ApiError::ProofOfStake(value) => POS_ERROR_OFFSET + u32::from(value),
            ApiError::User(value) => RESERVED_ERROR_MAX + 1 + u32::from(value),
        }
    }
}

impl From<u32> for ApiError {
    fn from(value: u32) -> ApiError {
        match value {
            1 => ApiError::None,
            2 => ApiError::MissingArgument,
            3 => ApiError::InvalidArgument,
            4 => ApiError::Deserialize,
            5 => ApiError::Read,
            6 => ApiError::ValueNotFound,
            7 => ApiError::ContractNotFound,
            8 => ApiError::GetKey,
            9 => ApiError::UnexpectedKeyVariant,
            10 => ApiError::UnexpectedContractRefVariant,
            11 => ApiError::InvalidPurseName,
            12 => ApiError::InvalidPurse,
            13 => ApiError::UpgradeContractAtURef,
            14 => ApiError::Transfer,
            15 => ApiError::NoAccessRights,
            16 => ApiError::CLTypeMismatch,
            17 => ApiError::EncodingExcessiveDiscriminants,
            18 => ApiError::EncodingEndOfSlice,
            19 => ApiError::EncodingLeftOverBytes,
            20 => ApiError::EncodingInvalidUtf8,
            21 => ApiError::EncodingInvalidBool,
            22 => ApiError::EncodingInvalidChar,
            23 => ApiError::EncodingInvalidTag,
            24 => ApiError::EncodingUnsupported,
            25 => ApiError::EncodingSizeLimit,
            26 => ApiError::EncodingSequenceMustHaveLength,
            27 => ApiError::EncodingCustom,
            28 => ApiError::MaxKeysLimit,
            29 => ApiError::DuplicateKey,
            30 => ApiError::PermissionDenied,
            31 => ApiError::MissingKey,
            32 => ApiError::ThresholdViolation,
            33 => ApiError::KeyManagementThreshold,
            34 => ApiError::DeploymentThreshold,
            35 => ApiError::InsufficientTotalWeight,
            36 => ApiError::InvalidSystemContract,
            37 => ApiError::PurseNotCreated,
            38 => ApiError::Unhandled,
            39 => ApiError::BufferTooSmall,
            40 => ApiError::HostBufferEmpty,
            41 => ApiError::HostBufferFull,
            42 => ApiError::AllocLayout,
            USER_ERROR_MIN..=USER_ERROR_MAX => ApiError::User(value as u16),
            POS_ERROR_MIN..=POS_ERROR_MAX => ApiError::ProofOfStake(value as u8),
            MINT_ERROR_MIN..=MINT_ERROR_MAX => ApiError::Mint(value as u8),
            _ => ApiError::Unhandled,
        }
    }
}

impl Debug for ApiError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ApiError::None => write!(f, "ApiError::None")?,
            ApiError::MissingArgument => write!(f, "ApiError::MissingArgument")?,
            ApiError::InvalidArgument => write!(f, "ApiError::InvalidArgument")?,
            ApiError::Deserialize => write!(f, "ApiError::Deserialize")?,
            ApiError::Read => write!(f, "ApiError::Read")?,
            ApiError::ValueNotFound => write!(f, "ApiError::ValueNotFound")?,
            ApiError::ContractNotFound => write!(f, "ApiError::ContractNotFound")?,
            ApiError::GetKey => write!(f, "ApiError::GetKey")?,
            ApiError::UnexpectedKeyVariant => write!(f, "ApiError::UnexpectedKeyVariant")?,
            ApiError::UnexpectedContractRefVariant => {
                write!(f, "ApiError::UnexpectedContractRefVariant")?
            }
            ApiError::InvalidPurseName => write!(f, "ApiError::InvalidPurseName")?,
            ApiError::InvalidPurse => write!(f, "ApiError::InvalidPurse")?,
            ApiError::UpgradeContractAtURef => write!(f, "ApiError::UpgradeContractAtURef")?,
            ApiError::Transfer => write!(f, "ApiError::Transfer")?,
            ApiError::NoAccessRights => write!(f, "ApiError::NoAccessRights")?,
            ApiError::CLTypeMismatch => write!(f, "ApiError::CLTypeMismatch")?,
            ApiError::EncodingExcessiveDiscriminants => {
                write!(f, "ApiError::EncodingExcessiveDiscriminants")?
            }
            ApiError::EncodingEndOfSlice => write!(f, "ApiError::EncodingEndOfSlice")?,
            ApiError::EncodingLeftOverBytes => write!(f, "ApiError::EncodingLeftOverBytes")?,
            ApiError::EncodingInvalidUtf8 => write!(f, "ApiError::EncodingInvalidUtf8")?,
            ApiError::EncodingInvalidBool => write!(f, "ApiError::EncodingInvalidBool")?,
            ApiError::EncodingInvalidChar => write!(f, "ApiError::EncodingInvalidChar")?,
            ApiError::EncodingInvalidTag => write!(f, "ApiError::EncodingInvalidTag")?,
            ApiError::EncodingUnsupported => write!(f, "ApiError::EncodingUnsupported")?,
            ApiError::EncodingSizeLimit => write!(f, "ApiError::EncodingSizeLimit")?,
            ApiError::EncodingSequenceMustHaveLength => {
                write!(f, "ApiError::EncodingSequenceMustHaveLength")?
            }
            ApiError::EncodingCustom => write!(f, "ApiError::EncodingCustom")?,
            ApiError::MaxKeysLimit => write!(f, "ApiError::MaxKeysLimit")?,
            ApiError::DuplicateKey => write!(f, "ApiError::DuplicateKey")?,
            ApiError::PermissionDenied => write!(f, "ApiError::PermissionDenied")?,
            ApiError::MissingKey => write!(f, "ApiError::MissingKey")?,
            ApiError::ThresholdViolation => write!(f, "ApiError::ThresholdViolation")?,
            ApiError::KeyManagementThreshold => write!(f, "ApiError::KeyManagementThreshold")?,
            ApiError::DeploymentThreshold => write!(f, "ApiError::DeploymentThreshold")?,
            ApiError::InsufficientTotalWeight => write!(f, "ApiError::InsufficientTotalWeight")?,
            ApiError::InvalidSystemContract => write!(f, "ApiError::InvalidSystemContract")?,
            ApiError::PurseNotCreated => write!(f, "ApiError::PurseNotCreated")?,
            ApiError::Unhandled => write!(f, "ApiError::Unhandled")?,
            ApiError::BufferTooSmall => write!(f, "ApiError::BufferTooSmall")?,
            ApiError::HostBufferEmpty => write!(f, "ApiError::HostBufferEmpty")?,
            ApiError::HostBufferFull => write!(f, "ApiError::HostBufferFull")?,
            ApiError::AllocLayout => write!(f, "ApiError::AllocLayout")?,
            ApiError::Mint(value) => write!(f, "ApiError::Mint({})", value)?,
            ApiError::ProofOfStake(value) => write!(f, "ApiError::ProofOfStake({})", value)?,
            ApiError::User(value) => write!(f, "ApiError::User({})", value)?,
        }
        write!(f, " [{}]", u32::from(*self))
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::User(value) => write!(f, "User error: {}", value),
            ApiError::Mint(value) => write!(f, "Mint error: {}", value),
            ApiError::ProofOfStake(value) => write!(f, "PoS error: {}", value),
            _ => <Self as Debug>::fmt(&self, f),
        }
    }
}

// This function is not intended to be used by third party crates.
#[doc(hidden)]
pub fn i32_from(result: Result<(), ApiError>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(error) => u32::from(error) as i32,
    }
}

/// Converts an `i32` to a `Result<(), ApiError>`, where `0` represents `Ok(())`, and all other
/// inputs are mapped to `Err(ApiError::<variant>)`.  The full list of mappings can be found in the
/// [docs for `ApiError`](ApiError#mappings).
pub fn result_from(value: i32) -> Result<(), ApiError> {
    match value {
        0 => Ok(()),
        _ => Err(ApiError::from(value as u32)),
    }
}

#[cfg(test)]
mod tests {
    use std::{i32, u16, u8};

    use super::*;

    fn round_trip(result: Result<(), ApiError>) {
        let code = i32_from(result);
        assert_eq!(result, result_from(code));
    }

    #[test]
    fn error() {
        assert_eq!(65_024_u32, ApiError::Mint(0).into()); // MINT_ERROR_OFFSET == 65,024
        assert_eq!(65_279_u32, ApiError::Mint(u8::MAX).into());
        assert_eq!(65_280_u32, ApiError::ProofOfStake(0).into()); // POS_ERROR_OFFSET == 65,280
        assert_eq!(65_535_u32, ApiError::ProofOfStake(u8::MAX).into());
        assert_eq!(65_536_u32, ApiError::User(0).into()); // u16::MAX + 1
        assert_eq!(131_071_u32, ApiError::User(u16::MAX).into()); // 2 * u16::MAX + 1

        assert_eq!("ApiError::GetKey [8]", &format!("{:?}", ApiError::GetKey));
        assert_eq!("ApiError::GetKey [8]", &format!("{}", ApiError::GetKey));
        assert_eq!(
            "ApiError::Mint(0) [65024]",
            &format!("{:?}", ApiError::Mint(0))
        );
        assert_eq!("Mint error: 0", &format!("{}", ApiError::Mint(0)));
        assert_eq!("Mint error: 255", &format!("{}", ApiError::Mint(u8::MAX)));
        assert_eq!(
            "ApiError::ProofOfStake(0) [65280]",
            &format!("{:?}", ApiError::ProofOfStake(0))
        );
        assert_eq!("PoS error: 0", &format!("{}", ApiError::ProofOfStake(0)));
        assert_eq!(
            "ApiError::ProofOfStake(255) [65535]",
            &format!("{:?}", ApiError::ProofOfStake(u8::MAX))
        );
        assert_eq!(
            "ApiError::User(0) [65536]",
            &format!("{:?}", ApiError::User(0))
        );
        assert_eq!("User error: 0", &format!("{}", ApiError::User(0)));
        assert_eq!(
            "ApiError::User(65535) [131071]",
            &format!("{:?}", ApiError::User(u16::MAX))
        );
        assert_eq!(
            "User error: 65535",
            &format!("{}", ApiError::User(u16::MAX))
        );

        assert_eq!(Err(ApiError::Unhandled), result_from(i32::MAX));
        assert_eq!(
            Err(ApiError::Unhandled),
            result_from(MINT_ERROR_OFFSET as i32 - 1)
        );
        assert_eq!(Err(ApiError::Unhandled), result_from(-1));
        assert_eq!(Err(ApiError::Unhandled), result_from(i32::MIN));

        round_trip(Ok(()));
        round_trip(Err(ApiError::None));
        round_trip(Err(ApiError::MissingArgument));
        round_trip(Err(ApiError::InvalidArgument));
        round_trip(Err(ApiError::Deserialize));
        round_trip(Err(ApiError::Read));
        round_trip(Err(ApiError::ValueNotFound));
        round_trip(Err(ApiError::ContractNotFound));
        round_trip(Err(ApiError::GetKey));
        round_trip(Err(ApiError::UnexpectedKeyVariant));
        round_trip(Err(ApiError::UnexpectedContractRefVariant));
        round_trip(Err(ApiError::InvalidPurseName));
        round_trip(Err(ApiError::InvalidPurse));
        round_trip(Err(ApiError::UpgradeContractAtURef));
        round_trip(Err(ApiError::Transfer));
        round_trip(Err(ApiError::NoAccessRights));
        round_trip(Err(ApiError::CLTypeMismatch));
        round_trip(Err(ApiError::EncodingExcessiveDiscriminants));
        round_trip(Err(ApiError::EncodingEndOfSlice));
        round_trip(Err(ApiError::EncodingLeftOverBytes));
        round_trip(Err(ApiError::EncodingInvalidUtf8));
        round_trip(Err(ApiError::EncodingInvalidBool));
        round_trip(Err(ApiError::EncodingInvalidChar));
        round_trip(Err(ApiError::EncodingInvalidTag));
        round_trip(Err(ApiError::EncodingUnsupported));
        round_trip(Err(ApiError::EncodingSizeLimit));
        round_trip(Err(ApiError::EncodingSequenceMustHaveLength));
        round_trip(Err(ApiError::EncodingCustom));
        round_trip(Err(ApiError::MaxKeysLimit));
        round_trip(Err(ApiError::DuplicateKey));
        round_trip(Err(ApiError::PermissionDenied));
        round_trip(Err(ApiError::MissingKey));
        round_trip(Err(ApiError::ThresholdViolation));
        round_trip(Err(ApiError::KeyManagementThreshold));
        round_trip(Err(ApiError::DeploymentThreshold));
        round_trip(Err(ApiError::InsufficientTotalWeight));
        round_trip(Err(ApiError::InvalidSystemContract));
        round_trip(Err(ApiError::PurseNotCreated));
        round_trip(Err(ApiError::Unhandled));
        round_trip(Err(ApiError::BufferTooSmall));
        round_trip(Err(ApiError::HostBufferEmpty));
        round_trip(Err(ApiError::HostBufferFull));
        round_trip(Err(ApiError::AllocLayout));
        round_trip(Err(ApiError::Mint(0)));
        round_trip(Err(ApiError::Mint(u8::MAX)));
        round_trip(Err(ApiError::ProofOfStake(0)));
        round_trip(Err(ApiError::ProofOfStake(u8::MAX)));
        round_trip(Err(ApiError::User(0)));
        round_trip(Err(ApiError::User(u16::MAX)));
    }
}
