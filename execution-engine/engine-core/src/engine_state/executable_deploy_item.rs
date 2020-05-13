use types::{bytesrepr, contracts::ContractVersion, HashAddr, RuntimeArgs};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExecutableDeployItem {
    ModuleBytes {
        module_bytes: Vec<u8>,
        // assumes implicit `call` noarg entrypoint
        args: Vec<u8>,
    },
    StoredContractByHash {
        hash: Vec<u8>,
        // TODO: add entrypoint -> once local key is replaced with hash, add_contract_version could
        // return hash
        args: Vec<u8>,
    },
    StoredContractByName {
        name: String,
        // TODO: add entrypoint name
        args: Vec<u8>,
    },
    StoredVersionedContractByName {
        name: String,             // named key storing contract metadata hash
        version: ContractVersion, // finds active version
        entry_point: String,      // finds header by entry point name
        args: Vec<u8>,
    },
    StoredVersionedContractByHash {
        hash: HashAddr,           // named key storing contract metadata hash
        version: ContractVersion, // finds active version
        entry_point: String,      // finds header by entry point name
        args: Vec<u8>,
    },
}

impl ExecutableDeployItem {
    pub fn take_args(self) -> Result<RuntimeArgs, bytesrepr::Error> {
        match self {
            ExecutableDeployItem::ModuleBytes { args, .. }
            | ExecutableDeployItem::StoredContractByHash { args, .. }
            | ExecutableDeployItem::StoredContractByName { args, .. } => {
                let vec: RuntimeArgs = bytesrepr::deserialize(args)?;
                Ok(vec.into())
            }
            ExecutableDeployItem::StoredVersionedContractByHash { args, .. }
            | ExecutableDeployItem::StoredVersionedContractByName { args, .. } => {
                let runtime_args: RuntimeArgs = bytesrepr::deserialize(args)?;
                Ok(runtime_args)
            }
        }
    }

    pub fn entry_point_name(&self) -> &str {
        match self {
            ExecutableDeployItem::StoredVersionedContractByName { entry_point, .. }
            | ExecutableDeployItem::StoredVersionedContractByHash { entry_point, .. } => {
                &entry_point
            }
            ExecutableDeployItem::ModuleBytes { .. }
            | ExecutableDeployItem::StoredContractByHash { .. }
            | ExecutableDeployItem::StoredContractByName { .. } => "call",
        }
    }
}
