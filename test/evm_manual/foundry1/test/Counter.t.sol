// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console2} from "forge-std/Test.sol";
import {StdInvariant} from "forge-std/StdInvariant.sol";
import {Counter} from "../src/Counter.sol";

contract CounterTest is Test {
    Counter public counter1;
    Counter public counter2;
    Counter public counter3;
    Counter public counter4;
    Counter public counter5;
    Counter public counter6;
    Counter public counter7;
    Counter public counter8;
    Counter public counter9;

    function setUp() external {
        counter2 = new Counter();
        targetContract(address(counter2));

        bytes4[] memory selectors = new bytes4[](1);
        selectors[0] = counter2.number.selector;
        FuzzSelector memory selector = FuzzSelector(address(counter2), selectors);
        targetSelector(selector);
    }

    function testCOUNTERE() external {
        assertEq(counter2.number(), 0);
    }
}
