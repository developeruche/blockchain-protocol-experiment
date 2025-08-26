// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";
import {Quorum} from "../src/Quorum.sol";
import {QuorumMockSuccess, QuorumMockFail} from "./mocks/PodMockQuorum.sol";

contract QuorumTest is Test {
    function testRequireQuorumSuccess() public {
        QuorumMockSuccess mock = new QuorumMockSuccess();
        address quorumAddr = Quorum.POD_REQUIRE_QUORUM;
        vm.etch(quorumAddr, address(mock).code);

        // Should not revert
        Quorum.requireQuorum(true, "Should not revert");
    }

    function testRequireQuorumFails() public {
        QuorumMockFail mock = new QuorumMockFail();
        address quorumAddr = Quorum.POD_REQUIRE_QUORUM;
        vm.etch(quorumAddr, address(mock).code);

        // Should revert with custom message
        vm.expectRevert(bytes("Should fail quorum"));
        Quorum.requireQuorum(false, "Should fail quorum");
    }
}
