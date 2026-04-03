// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import {AutomateRegistry} from "../src/AutomateRegistry.sol";
import {MockAutomatable} from "../test/MockAutomatable.sol";

contract DeployAnvil is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployerPrivateKey);

        // 1. Deploy Registry
        AutomateRegistry registry = new AutomateRegistry();
        console.log("AutomateRegistry deployed at:", address(registry));

        // 2. Deploy Mock Automatable
        MockAutomatable mock = new MockAutomatable(address(registry));
        console.log("MockAutomatable deployed at:", address(mock));

        // 3. Add deployer as keeper
        registry.addKeeper(deployerAddress);
        console.log("Added keeper:", deployerAddress);

        // 4. Register a dummy job
        bytes memory data = abi.encode(uint256(42));
        bytes32 jobId = registry.registerJob(address(mock), data);

        console.log("----- CONFIG VARS -----");
        console.log("automate_contract = '%s'", address(registry));
        console.log("calldata = '%s'", vm.toString(data));
        console.log("jobId = '%s'", vm.toString(jobId));

        vm.stopBroadcast();
    }
}
