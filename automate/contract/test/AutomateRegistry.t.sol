// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {AutomateRegistry} from "../src/AutomateRegistry.sol";
import {MockAutomatable} from "./MockAutomatable.sol";

contract AutomateRegistryTest is Test {
    AutomateRegistry public registry;
    MockAutomatable public target;

    address public owner = makeAddr("owner");
    address public keeper = makeAddr("keeper");
    address public jobOwner = makeAddr("jobOwner");
    address public rando = makeAddr("rando");

    bytes public defaultData = abi.encode(uint256(42));

    function setUp() public {
        vm.prank(owner);
        registry = new AutomateRegistry();

        target = new MockAutomatable(address(registry));

        vm.prank(owner);
        registry.addKeeper(keeper);
    }

    // ==================
    // Registration Tests
    // ==================

    function test_registerJob_Success() public {
        vm.startPrank(jobOwner);

        bytes32 expectedJobId = registry.computeJobId(jobOwner, address(target), defaultData);

        vm.expectEmit(true, true, true, true);
        emit AutomateRegistry.JobRegistered(expectedJobId, jobOwner, address(target), defaultData);

        bytes32 jobId = registry.registerJob(address(target), defaultData);

        assertEq(jobId, expectedJobId);

        AutomateRegistry.Job memory job = registry.getJob(jobId);
        assertEq(job.owner, jobOwner);
        assertEq(job.target, address(target));
        assertEq(job.callData, defaultData);
        assertEq(uint256(job.status), uint256(AutomateRegistry.JobStatus.Active));
        assertEq(job.executionCount, 0);
        assertEq(job.lastExecutedAt, 0);
        assertEq(job.createdAt, block.timestamp);

        assertTrue(registry.isJobActive(jobId));
        vm.stopPrank();
    }

    function test_registerJob_ZeroAddressTarget() public {
        vm.prank(jobOwner);
        vm.expectRevert(AutomateRegistry.ZeroAddress.selector);
        registry.registerJob(address(0), defaultData);
    }

    function test_registerJob_AlreadyExists() public {
        vm.startPrank(jobOwner);

        // Registering the exact same job in the same block will generate the same jobId
        bytes32 jobId = registry.registerJob(address(target), defaultData);
        assertTrue(jobId != bytes32(0));

        vm.expectRevert(AutomateRegistry.JobAlreadyExists.selector);
        registry.registerJob(address(target), defaultData);

        vm.stopPrank();
    }

    // ====================
    // Execution Tests
    // ====================

    function test_execute_Success() public {
        bytes32 jobId = _registerDefaultJob();

        vm.warp(block.timestamp + 100);

        vm.prank(keeper);

        vm.expectEmit(true, true, false, true);
        emit AutomateRegistry.JobExecuted(jobId, keeper, 1, uint64(block.timestamp));

        registry.execute(jobId);

        assertEq(target.callCount(), 1);
        assertEq(target.lastJobId(), jobId);
        assertEq(target.lastData(), defaultData);

        AutomateRegistry.Job memory job = registry.getJob(jobId);
        assertEq(job.executionCount, 1);
        assertEq(job.lastExecutedAt, block.timestamp);
    }

    function test_execute_NotKeeper() public {
        bytes32 jobId = _registerDefaultJob();

        vm.prank(rando);
        vm.expectRevert(AutomateRegistry.NotKeeper.selector);
        registry.execute(jobId);
    }

    function test_execute_JobNotFound() public {
        bytes32 badJobId = keccak256("bad job");

        vm.prank(keeper);
        vm.expectRevert(AutomateRegistry.JobNotFound.selector);
        registry.execute(badJobId);
    }

    function test_execute_JobNotActive() public {
        bytes32 jobId = _registerDefaultJob();

        vm.prank(jobOwner);
        registry.pauseJob(jobId);

        vm.prank(keeper);
        vm.expectRevert(AutomateRegistry.JobNotActive.selector);
        registry.execute(jobId);
    }

    function test_execute_TargetReverts_BubblesUp() public {
        bytes32 jobId = _registerDefaultJob();
        target.setRevert(true);

        vm.prank(keeper);
        // Note: IAutomatable revert might not bubble up nicely or it bubbles up as TargetCallFailed
        vm.expectRevert(MockAutomatable.ForcedFailure.selector);
        registry.execute(jobId);
    }

    function test_execute_TargetReturnsFalse_BubblesUp() public {
        bytes32 jobId = _registerDefaultJob();
        target.setReturnFalse(true);

        vm.prank(keeper);
        vm.expectRevert(AutomateRegistry.TargetCallFailed.selector);
        registry.execute(jobId);
    }

    // ====================
    // Pause / Resume Tests
    // ====================

    function test_pauseJob_Success() public {
        bytes32 jobId = _registerDefaultJob();

        vm.prank(jobOwner);
        vm.expectEmit(true, false, false, false);
        emit AutomateRegistry.JobPaused(jobId);
        registry.pauseJob(jobId);

        AutomateRegistry.Job memory job = registry.getJob(jobId);
        assertEq(uint256(job.status), uint256(AutomateRegistry.JobStatus.Paused));
        assertFalse(registry.isJobActive(jobId));
    }

    function test_pauseJob_NotJobOwner() public {
        bytes32 jobId = _registerDefaultJob();

        vm.prank(rando);
        vm.expectRevert(AutomateRegistry.NotJobOwner.selector);
        registry.pauseJob(jobId);
    }

    function test_pauseJob_AlreadyCancelled() public {
        bytes32 jobId = _registerDefaultJob();

        vm.startPrank(jobOwner);
        registry.cancelJob(jobId);

        vm.expectRevert(AutomateRegistry.JobAlreadyCancelled.selector);
        registry.pauseJob(jobId);
        vm.stopPrank();
    }

    function test_resumeJob_Success() public {
        bytes32 jobId = _registerDefaultJob();

        vm.startPrank(jobOwner);
        registry.pauseJob(jobId);

        vm.expectEmit(true, false, false, false);
        emit AutomateRegistry.JobResumed(jobId);
        registry.resumeJob(jobId);
        vm.stopPrank();

        AutomateRegistry.Job memory job = registry.getJob(jobId);
        assertEq(uint256(job.status), uint256(AutomateRegistry.JobStatus.Active));
        assertTrue(registry.isJobActive(jobId));
    }

    function test_resumeJob_NotJobOwner() public {
        bytes32 jobId = _registerDefaultJob();

        vm.prank(jobOwner);
        registry.pauseJob(jobId);

        vm.prank(rando);
        vm.expectRevert(AutomateRegistry.NotJobOwner.selector);
        registry.resumeJob(jobId);
    }

    function test_resumeJob_AlreadyCancelled() public {
        bytes32 jobId = _registerDefaultJob();

        vm.startPrank(jobOwner);
        registry.cancelJob(jobId);

        vm.expectRevert(AutomateRegistry.JobAlreadyCancelled.selector);
        registry.resumeJob(jobId);
        vm.stopPrank();
    }

    // ====================
    // Cancel Tests
    // ====================

    function test_cancelJob_Success() public {
        bytes32 jobId = _registerDefaultJob();

        vm.prank(jobOwner);
        vm.expectEmit(true, false, false, false);
        emit AutomateRegistry.JobCancelled(jobId);
        registry.cancelJob(jobId);

        AutomateRegistry.Job memory job = registry.getJob(jobId);
        assertEq(uint256(job.status), uint256(AutomateRegistry.JobStatus.Cancelled));
        assertFalse(registry.isJobActive(jobId));
    }

    function test_cancelJob_NotJobOwner() public {
        bytes32 jobId = _registerDefaultJob();

        vm.prank(rando);
        vm.expectRevert(AutomateRegistry.NotJobOwner.selector);
        registry.cancelJob(jobId);
    }

    // ====================
    // Keeper Admin Tests
    // ====================

    function test_addKeeper_Success() public {
        address newKeeper = makeAddr("newKeeper");

        vm.prank(owner);
        vm.expectEmit(true, false, false, false);
        emit AutomateRegistry.KeeperAdded(newKeeper);
        registry.addKeeper(newKeeper);

        assertTrue(registry.isKeeper(newKeeper));
    }

    function test_addKeeper_NotOwner() public {
        address newKeeper = makeAddr("newKeeper");

        vm.prank(rando);
        vm.expectRevert(AutomateRegistry.NotOwner.selector);
        registry.addKeeper(newKeeper);
    }

    function test_addKeeper_ZeroAddress() public {
        vm.prank(owner);
        vm.expectRevert(AutomateRegistry.ZeroAddress.selector);
        registry.addKeeper(address(0));
    }

    function test_removeKeeper_Success() public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, false);
        emit AutomateRegistry.KeeperRemoved(keeper);
        registry.removeKeeper(keeper);

        assertFalse(registry.isKeeper(keeper));
    }

    function test_removeKeeper_NotOwner() public {
        vm.prank(rando);
        vm.expectRevert(AutomateRegistry.NotOwner.selector);
        registry.removeKeeper(keeper);
    }

    // ====================
    // Internal Helpers
    // ====================

    function _registerDefaultJob() internal returns (bytes32) {
        vm.prank(jobOwner);
        return registry.registerJob(address(target), defaultData);
    }
}
