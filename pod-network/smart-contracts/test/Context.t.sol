// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";
import {Context} from "../src/utils/Context.sol";
import {PodTxInfoMock, PodTxInfoWrongMock} from "./mocks/PodTxInfoMock.sol";

contract ContextTest is Test {
    function testGetTxInfo() public {
        // Deploy the mock
        PodTxInfoMock mock = new PodTxInfoMock();

        // Overwrite POD_TX_INFO address with mock code
        address podTxInfoAddr = Context.POD_TX_INFO;
        vm.etch(podTxInfoAddr, address(mock).code);

        // Call getTxInfo and check results
        Context.TxInfo memory info = Context.getTxInfo();
        assertEq(info.nonce, 42);
        assertEq(info.txHash, bytes32(uint256(123)));
    }

    function testGetTxInfoWrongDataFails() public {
        // Deploy the wrong mock
        PodTxInfoWrongMock wrongMock = new PodTxInfoWrongMock();

        // Overwrite POD_TX_INFO address with wrong mock code
        address podTxInfoAddr = Context.POD_TX_INFO;
        vm.etch(podTxInfoAddr, address(wrongMock).code);

        // Call getTxInfo and expect wrong data
        Context.TxInfo memory info = Context.getTxInfo();

        // These assertions will fail if data is wrong (this is allowed in tests)
        assertEq(info.nonce, 99, "Nonce should be 42");
        assertEq(info.txHash, bytes32(uint256(456)), "TxHash should be 123");
    }
}
