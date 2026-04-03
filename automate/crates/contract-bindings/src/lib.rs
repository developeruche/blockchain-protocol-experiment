use alloy::primitives::Address;
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract AutomateRegistry {
        function execute(bytes32 jobId) external;
        
        function getJob(bytes32 jobId) external view returns (
            address owner,
            address target,
            bytes callData,
            uint8 status,
            uint64 executionCount,
            uint64 lastExecutedAt,
            uint64 createdAt
        );
        
        function isKeeper(address addr) external view returns (bool);
        function isJobActive(bytes32 jobId) external view returns (bool);

        event JobExecuted(bytes32 indexed jobId, address indexed keeper, uint64 executionCount, uint64 timestamp);
    }
}

#[derive(Clone)]
pub struct AutomateContract<P> {
    pub address: Address,
    pub http_provider: P,
}

impl<P> AutomateContract<P> {
    pub fn new(address: Address, http_provider: P) -> Self {
        Self { address, http_provider }
    }
}
