// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract MutaoSwapHistory {
    struct SwapRecord {
        address previousOwner;
        address newOwner;
        uint256 timestamp;
        string message;
        string itemName;
        uint256 swapCount;
    }

    mapping(string => SwapRecord[]) public history;
    mapping(string => uint256) public swapCounts;

    event SwapRecorded(
        string indexed itemId,
        address indexed previousOwner,
        address indexed newOwner,
        uint256 timestamp,
        string message
    );

    function recordSwap(
        string memory _itemId,
        address _previousOwner,
        address _newOwner,
        string memory _message,
        string memory _itemName
    ) external {
        require(_previousOwner != address(0), "invalid previous owner");
        require(_newOwner != address(0), "invalid new owner");
        require(_previousOwner != _newOwner, "owners must differ");

        uint256 count = swapCounts[_itemId] + 1;
        swapCounts[_itemId] = count;

        history[_itemId].push(SwapRecord({
            previousOwner: _previousOwner,
            newOwner: _newOwner,
            timestamp: block.timestamp,
            message: _message,
            itemName: _itemName,
            swapCount: count
        }));

        emit SwapRecorded(_itemId, _previousOwner, _newOwner, block.timestamp, _message);
    }

    function getHistory(string memory _itemId) external view returns (SwapRecord[] memory) {
        return history[_itemId];
    }

    function getSwapCount(string memory _itemId) external view returns (uint256) {
        return swapCounts[_itemId];
    }
}
