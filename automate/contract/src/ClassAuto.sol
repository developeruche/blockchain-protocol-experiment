// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IAutomatable} from "./IAutomatable.sol";

enum AutomateType {
    Timeout,
    Heartbeat
}

struct AutoParams {
    AutomateType automateType;
    uint256 value;
}

contract ClassAuto is IAutomatable {
    address immutable REGISTRY;
    uint256 public timeout;
    uint256 public heartbeat;

    event IncreaseTimeout(uint256 _timeout);
    event IncreaseHeartbeat(uint256 _heartbeat);

    constructor(address _registry) {
        REGISTRY = _registry;
    }

    function increaseTimeout(uint256 _timeout) public {
        timeout += _timeout;
        emit IncreaseTimeout(_timeout);
    }

    function increaseHeartbeat(uint256 _heartbeat) public {
        heartbeat += _heartbeat;
        emit IncreaseHeartbeat(_heartbeat);
    }

    function performAutomation(bytes32 _jobId, bytes calldata data) external override returns (bool success) {
        require(msg.sender == REGISTRY, "Only registry can call this function");
        AutoParams memory autoParams = abi.decode(data, (AutoParams));

        if (autoParams.automateType == AutomateType.Timeout) {
            increaseTimeout(autoParams.value);
        } else if (autoParams.automateType == AutomateType.Heartbeat) {
            increaseHeartbeat(autoParams.value);
        }

        return true;
    }
}
