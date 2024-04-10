// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.15;

import "../../../solidity_utils/lib.sol";

contract main {
    address private owner;

    function destruct() external {
        selfdestruct(payable(msg.sender));
    }

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "not owner");
        _;
    }

    function admin_destruct() onlyOwner external {
        selfdestruct(payable(msg.sender));
    }

    function getOwner() public view returns (address) {
        return owner;
    }
}

//owner：这是一个私有地址变量，用于存储合约的所有者。
//constructor()：这是一个构造函数，当合约被部署时会被调用。它将msg.sender（部署合约的地址）设置为所有者。
//onlyOwner：这是一个修饰符，用于限制只有所有者才能调用的函数。如果msg.sender（调用函数的地址）不是所有者，它将抛出一个错误。
//destruct()：这是一个外部函数，任何人都可以调用。它会销毁合约，并将合约余额发送给调用者。
//admin_destruct()：这是一个只有所有者才能调用的外部函数。它会销毁合约，并将合约余额发送给所有者。
//getOwner()：这是一个公共函数，任何人都可以调用。它返回合约的所有者地址。