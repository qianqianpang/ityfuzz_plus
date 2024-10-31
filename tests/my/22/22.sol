pragma solidity 0.6.5;
pragma experimental ABIEncoderV2;

interface ERC20 {

    function approve(address, uint256) external returns (bool);

    function transfer(address, uint256) external returns (bool);

    function transferFrom(address, address, uint256) external returns (bool);

    function name() external view returns (string memory);

    function symbol() external view returns (string memory);

    function decimals() external view returns (uint8);

    function totalSupply() external view returns (uint256);

    function balanceOf(address) external view returns (uint256);

}

interface ProtocolAdapter {



    /**

     * @dev MUST return "Asset" or "Debt".

     * SHOULD be implemented by the public constant state variable.

     */

    function adapterType() external pure returns (string memory);



    /**

     * @dev MUST return token type (default is "ERC20").

     * SHOULD be implemented by the public constant state variable.

     */

    function tokenType() external pure returns (string memory);



    /**

     * @dev MUST return amount of the given token locked on the protocol by the given account.

     */

    function getBalance(address token, address account) external view returns (uint256);

}

contract TokenSetsV2Adapter is ProtocolAdapter {



    string public constant override adapterType = "Asset";



    string public constant override tokenType = "SetToken V2";



    /**

     * @return Amount of SetTokens held by the given account.

     * @param token Address of the SetToken contract.

     * @dev Implementation of ProtocolAdapter interface function.

     */

    function getBalance(address token, address account) external view override returns (uint256) {

        return ERC20(token).balanceOf(account);

    }

}
