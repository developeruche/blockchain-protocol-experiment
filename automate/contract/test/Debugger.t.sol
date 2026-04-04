// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import {AutomateRegistry} from "../src/AutomateRegistry.sol";
import {MockAutomatable} from "./MockAutomatable.sol";

contract DebuggerTest is Test {
    AutomateRegistry registry;
    address keeper = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    function setUp() public {
        registry = new AutomateRegistry();
        registry.addKeeper(keeper);
    }

    function testExecuteTrace() public {
        vm.startPrank(keeper);

        bytes memory data = abi.encode(uint256(42));
        MockAutomatable mock = new MockAutomatable(address(registry));
        bytes32 jobId = registry.registerJob(address(mock), data);

        bool active = registry.isJobActive(jobId);
        console.log("Is active locally deployed:", active);

        registry.execute(jobId);
        console.log("Execution successful locally!");
        vm.stopPrank();
    }
}
