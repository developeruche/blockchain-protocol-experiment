// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IAutomatable} from "./IAutomatable.sol";

/// @title AutomateRegistry
/// @notice Permissionless job registry. Anyone can register a job.
///         Only approved keepers may execute jobs.
///         The Rust keeper never touches user contracts directly —
///         it always routes through execute() here.
contract AutomateRegistry {
    enum JobStatus {
        Active,
        Paused,
        Cancelled
    }

    struct Job {
        address owner; // who registered the job
        address target; // the IAutomatable contract to call
        bytes callData; // forwarded verbatim to performAutomation()
        JobStatus status;
        uint64 executionCount;
        uint64 lastExecutedAt; // block.timestamp of last successful execution
        uint64 createdAt;
    }

    address public owner;
    /// jobId → Job
    mapping(bytes32 => Job) private _jobs;
    /// address → is approved keeper
    mapping(address => bool) private _keepers;

    event JobRegistered(bytes32 indexed jobId, address indexed jobOwner, address indexed target, bytes callData);
    event JobExecuted(bytes32 indexed jobId, address indexed keeper, uint64 executionCount, uint64 timestamp);
    event JobPaused(bytes32 indexed jobId);
    event JobResumed(bytes32 indexed jobId);
    event JobCancelled(bytes32 indexed jobId);
    event KeeperAdded(address indexed keeper);
    event KeeperRemoved(address indexed keeper);

    error NotOwner();
    error NotJobOwner();
    error NotKeeper();
    error JobNotFound();
    error JobNotActive();
    error JobAlreadyCancelled();
    error TargetCallFailed();
    error ZeroAddress();
    error JobAlreadyExists();


    modifier onlyOwner() {
        _onlyOwner();
        _;
    }

    function _onlyOwner() internal view {
        if (msg.sender != owner) revert NotOwner();
    }

    modifier onlyKeeper() {
        _onlyKeeper();
        _;
    }

    function _onlyKeeper() internal view {
        if (!_keepers[msg.sender]) revert NotKeeper();
    }

    modifier onlyJobOwner(bytes32 jobId) {
        _onlyJobOwner(jobId);
        _;
    }

    function _onlyJobOwner(bytes32 jobId) internal view {
        if (_jobs[jobId].owner != msg.sender) revert NotJobOwner();
    }

    modifier jobExists(bytes32 jobId) {
        _jobExists(jobId);
        _;
    }

    function _jobExists(bytes32 jobId) internal view {
        if (_jobs[jobId].createdAt == 0) revert JobNotFound();
    }


    constructor() {
        owner = msg.sender;
    }


    /// @notice Register a new automation job.
    /// @param target    The contract that implements IAutomatable.
    /// @param callData  Arbitrary bytes forwarded to performAutomation().
    ///                  Use abi.encode() to pack your params off-chain.
    /// @return jobId    A deterministic ID derived from owner + target + callData + block.
    ///                  Store this — it's how you reference the job everywhere.
    function registerJob(address target, bytes calldata callData) external returns (bytes32 jobId) {
        if (target == address(0)) revert ZeroAddress();

        jobId = _computeJobId(msg.sender, target, callData);

        if (_jobs[jobId].createdAt != 0) revert JobAlreadyExists();

        _jobs[jobId] = Job({
            owner: msg.sender,
            target: target,
            callData: callData,
            status: JobStatus.Active,
            executionCount: 0,
            lastExecutedAt: 0,
            createdAt: uint64(block.timestamp)
        });

        emit JobRegistered(jobId, msg.sender, target, callData);
    }


    /// @notice Execute a job. Only callable by an approved keeper.
    ///         Calls target.performAutomation(jobId, callData).
    ///         Reverts the whole tx if the target call fails —
    ///         this means the keeper knows immediately without log parsing.
    /// @param jobId  The job to execute.
    function execute(bytes32 jobId) external onlyKeeper jobExists(jobId) {
        Job storage job = _jobs[jobId];

        if (job.status != JobStatus.Active) revert JobNotActive();

        // State update BEFORE external call
        job.executionCount += 1;
        job.lastExecutedAt = uint64(block.timestamp);

        // External call — forward original callData
        bool success = IAutomatable(job.target).performAutomation(jobId, job.callData);

        if (!success) revert TargetCallFailed();

        emit JobExecuted(jobId, msg.sender, job.executionCount, uint64(block.timestamp));
    }


    function pauseJob(bytes32 jobId) external jobExists(jobId) onlyJobOwner(jobId) {
        Job storage job = _jobs[jobId];
        if (job.status == JobStatus.Cancelled) revert JobAlreadyCancelled();
        job.status = JobStatus.Paused;
        emit JobPaused(jobId);
    }

    function resumeJob(bytes32 jobId) external jobExists(jobId) onlyJobOwner(jobId) {
        Job storage job = _jobs[jobId];
        if (job.status == JobStatus.Cancelled) revert JobAlreadyCancelled();
        job.status = JobStatus.Active;
        emit JobResumed(jobId);
    }

    /// @notice Permanently cancel a job. Cannot be undone.
    function cancelJob(bytes32 jobId) external jobExists(jobId) onlyJobOwner(jobId) {
        _jobs[jobId].status = JobStatus.Cancelled;
        emit JobCancelled(jobId);
    }


    function addKeeper(address keeper) external onlyOwner {
        if (keeper == address(0)) revert ZeroAddress();
        _keepers[keeper] = true;
        emit KeeperAdded(keeper);
    }

    function removeKeeper(address keeper) external onlyOwner {
        _keepers[keeper] = false;
        emit KeeperRemoved(keeper);
    }


    function getJob(bytes32 jobId) external view jobExists(jobId) returns (Job memory) {
        return _jobs[jobId];
    }

    function isKeeper(address addr) external view returns (bool) {
        return _keepers[addr];
    }

    function isJobActive(bytes32 jobId) external view returns (bool) {
        return _jobs[jobId].status == JobStatus.Active;
    }

    /// @notice Deterministically compute a jobId off-chain before registering.
    function computeJobId(address jobOwner, address target, bytes calldata callData) external view returns (bytes32) {
        return _computeJobId(jobOwner, target, callData);
    }
    
    function _computeJobId(address jobOwner, address target, bytes memory callData) internal view returns (bytes32) {
        // block.timestamp in the seed makes collisions on re-registration
        // effectively impossible without making jobId predictable from off-chain.
        bytes memory data = abi.encodePacked(jobOwner, target, callData, block.timestamp, block.chainid);
        bytes32 result;
        assembly {
            result := keccak256(add(data, 0x20), mload(data))
        }
        return result;
    }
}
