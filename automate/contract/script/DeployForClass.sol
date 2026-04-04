// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import {AutomateRegistry} from "../src/AutomateRegistry.sol";
import {ClassAuto, AutoParams, AutomateType} from "../src/ClassAuto.sol";

contract DeployForClass is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployerPrivateKey);

        // 1. Deploy Registry
        AutomateRegistry registry = new AutomateRegistry();
        console.log("AutomateRegistry deployed at:", address(registry));

        // 2. Deploy ClassAuto
        ClassAuto classAuto = new ClassAuto(address(registry));
        console.log("ClassAuto deployed at:", address(classAuto));

        // 3. Add deployer as keeper
        registry.addKeeper(deployerAddress);
        console.log("Added keeper:", deployerAddress);

        // 4. Register timeout job
        AutoParams memory params = AutoParams({automateType: AutomateType.Timeout, value: 120});
        bytes memory data = abi.encode(params);
        bytes32 jobId = registry.registerJob(address(classAuto), data);

        // 5. Register heartbeat job
        AutoParams memory params2 = AutoParams({automateType: AutomateType.Heartbeat, value: 60});
        bytes memory data2 = abi.encode(params2);
        bytes32 jobId2 = registry.registerJob(address(classAuto), data2);

        console.log("----- CONFIG VARS -----");
        console.log("automate_contract = '%s'", address(registry));
        console.log("calldata = '%s'", vm.toString(data));
        console.log("jobId = '%s'", vm.toString(jobId));
        console.log("calldata2 = '%s'", vm.toString(data2));
        console.log("jobId2 = '%s'", vm.toString(jobId2));

        vm.stopBroadcast();
    }
}
