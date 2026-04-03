// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import {AutomateRegistry} from "../src/AutomateRegistry.sol";

contract TestDump is Script {
    function run() external {
        AutomateRegistry reg = AutomateRegistry(0xc5a5C42992dECbae36851359345FE25997F5C42d);
        bytes32 jid = 0xa7b73adf304f688eafb0bf24cac4a1906199043fbf80091a30d09401457fdd50;

        bool active = reg.isJobActive(jid);
        console.log("isJobActive:", active);

        address owner = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

        vm.startBroadcast(owner);
        // Let's actually execute it via a script!
        reg.execute(jid);
        console.log("Executed successfully!");
        vm.stopBroadcast();
    }
}
