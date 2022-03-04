use ethers::prelude::Provider;
use revm::{
    db::{DatabaseRef, EmptyDB},
    Env, SpecId,
};

use super::{
    fork::{SharedBackend, SharedMemCache},
    Executor,
};

#[derive(Default)]
pub struct ExecutorBuilder {
    /// Whether or not cheatcodes are enabled
    cheatcodes: bool,
    /// Whether or not the FFI cheatcode is enabled
    ffi: bool,
    /// The execution environment configuration.
    config: Env,
    fork: Option<Fork>,
}

#[derive(Clone, Debug)]
pub struct Fork {
    // todo: cache path
    /// The URL to a node for fetching remote state
    pub url: String,
    /// The block to fork against
    pub pin_block: Option<u64>,
}

pub enum Backend {
    Simple(EmptyDB),
    Forked(SharedBackend),
}

impl Backend {
    /// Instantiates a new backend union based on whether there was or not a fork url specified
    fn new(fork: Option<Fork>) -> Self {
        if let Some(fork) = fork {
            let provider = Provider::try_from(fork.url).unwrap();
            // TOOD: Add reading cache from disk
            let backend = SharedBackend::new(
                provider,
                SharedMemCache::default(),
                fork.pin_block.map(Into::into),
            );
            Backend::Forked(backend)
        } else {
            Backend::Simple(EmptyDB())
        }
    }
}

use ethers::types::{H160, H256, U256};
use revm::AccountInfo;

impl DatabaseRef for Backend {
    fn block_hash(&self, number: U256) -> H256 {
        match self {
            Backend::Simple(inner) => inner.block_hash(number),
            Backend::Forked(inner) => inner.block_hash(number),
        }
    }

    fn basic(&self, address: H160) -> AccountInfo {
        match self {
            Backend::Simple(inner) => inner.basic(address),
            Backend::Forked(inner) => inner.basic(address),
        }
    }

    fn code_by_hash(&self, address: H256) -> bytes::Bytes {
        match self {
            Backend::Simple(inner) => inner.code_by_hash(address),
            Backend::Forked(inner) => inner.code_by_hash(address),
        }
    }

    fn storage(&self, address: H160, index: U256) -> U256 {
        match self {
            Backend::Simple(inner) => inner.storage(address, index),
            Backend::Forked(inner) => inner.storage(address, index),
        }
    }
}

impl ExecutorBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { cheatcodes: false, ffi: false, config: Env::default(), fork: None }
    }

    /// Enables cheatcodes on the executor.
    #[must_use]
    pub fn with_cheatcodes(mut self, ffi: bool) -> Self {
        self.cheatcodes = true;
        self.ffi = ffi;
        self
    }

    #[must_use]
    pub fn with_spec(mut self, spec: SpecId) -> Self {
        self.config.cfg.spec_id = spec;
        self
    }

    /// Configure the execution environment (gas limit, chain spec, ...)
    #[must_use]
    pub fn with_config(mut self, config: Env) -> Self {
        self.config = config;
        self
    }

    /// Configure the executor's forking mode
    #[must_use]
    pub fn with_fork(mut self, fork: Fork) -> Self {
        self.fork = Some(fork);
        self
    }

    /// Builds the executor as configured.
    pub fn build(self) -> Executor<Backend> {
        let db = Backend::new(self.fork);
        Executor::new(db, self.config)
    }

    // TODO: add with_traces
    // TODO: add with_debug(ger?)
    // TODO: add forked
}
