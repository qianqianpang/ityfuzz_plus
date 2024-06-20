/// This file is auto-generated by Scribble and shouldn't be edited directly.
/// Use --disarm prior to make any changes.
pragma solidity 0.8.19;

//contract Foo {
//    /// #if_succeeds {:msg "P1"} y == x + 2;
//    function inc(uint x) public pure returns (uint y) {
//        return x+1;
//    }
//}

contract Foo {
    function inc(uint x) public returns (uint y) {
        y = _original_Foo_inc(x);
        unchecked {
            if (!(y == (x + 2))) {
                emit __ScribbleUtilsLib__15.AssertionFailed("000318:0068:000 0: P1");
            }
        }
    }

    function _original_Foo_inc(uint x) private pure returns (uint y) {
        return x + 1;
    }
}

library __ScribbleUtilsLib__15 {
    event AssertionFailed(string message);

    event AssertionFailedData(int eventId, bytes encodingData);

    function assertionFailed(string memory arg_0) internal {
        emit AssertionFailed(arg_0);
    }

    function assertionFailedData(int arg_0, bytes memory arg_1) internal {
        emit AssertionFailedData(arg_0, arg_1);
    }

    function isInContract() internal returns (bool res) {
        assembly {
            res := sload(0x5f0b92cf9616afdee4f4136f66393f1343b027f01be893fa569eb2e2b667a40c)
        }
    }

    function setInContract(bool v) internal {
        assembly {
            sstore(0x5f0b92cf9616afdee4f4136f66393f1343b027f01be893fa569eb2e2b667a40c, v)
        }
    }
}