### EVM 执行器: revm 

还利用 revm 的解释器钩子来执行动态仪表、收集dataflow和comparison信息，并进行快速快照


### 数据集：

三个数据集（B1、B2 和 B3）来评估我们的工具。
B1（摘自 VERISMART ）包含 57 个支持 ERC20 标准接口（即代币）的智能合约。
B2 和 B3 分别包含 72 份和 500 份从以太坊链中抓取的智能合约。
几个projects

### #定制oracle

手动实现Oracle trait的两个方法
transition方法用于在每个阶段的开始时更新oracle的状态
而oracle方法则用于在每个阶段的结束时生成bug报告。

### onchain 和  RPC
1)格式：
`ityfuzz evm\
-t [TARGET_ADDR]\
--onchain-block-number [BLOCK]\
-c [CHAIN_TYPE]\
--onchain-etherscan-api-key [Etherscan API Key] # (Optional) specify your etherscan api key`
2)参数含义
-t [TARGET_ADDR]: specify the target contract
--onchain-block-number [BLOCK]: fork the chain at block number [BLOCK]即将要分叉的块号。
-c [CHAIN_TYPE]: specify the chain(BSC,POLYGON,ETHEREUM,etc.)
-f: (Optional) allow attack to get flashloan
`-onchain-etherscan-api-key` 参数用来指定以太坊区块链的区块浏览器服务的 API 密钥。这个 API 密钥允许程序通过以太坊区块浏览器服务进行区块链数据的查询和访问，比如检索交易历史、智能合约信息等。API 密钥是对区块浏览器服务进行身份验证的一种方式，它确保了只有授权的用户才能访问区块链数据。

3)RPC
Ityfuzz 将优先读取 `ETH_RPC_URL` 环境变量作为 RPC 地址，如果没有设置，将使用内置的公共 RPC 地址。
RPC地址（Remote Procedure Call Address）是一个网络地址，指向一个运行Ethereum节点的服务器，这个节点提供了一个JSON-RPC接口，可以用来查询区块链的状态，发送交易，调用智能合约等。
例如，一个典型的Ethereum RPC地址可能看起来像这样：`http://localhost:8545`。这个地址指向运行在本地机器上的Ethereum节点，该节点监听8545端口上的JSON-RPC请求。
在命令行参数中指定RPC地址，例如：`ityfuzz evm -o -t [TARGET_ADDR] --onchain-block-number [BLOCK] -c [CHAIN_TYPE] --onchain-etherscan-api-key [Etherscan API Key] --rpc-url [RPC_URL]`
4)区别
二者都可以用于查询区块链的状态、发送交易、调用智能合约等操作，但是它们的作用对象和机制略有不同：
- `--onchain-etherscan-api-key` 是用于访问以太坊区块浏览器服务的 API 密钥。它允许程序通过区块浏览器的 API 接口来查询区块链数据，而不是直接连接到以太坊节点。
- `ETH_RPC_URL` 是指定连接到运行在特定服务器上的以太坊节点的 RPC 地址。通过这个地址，程序可以直接与以太坊节点通信，发送交易、查询状态、调用智能合约等。
的使用场景略有不同。
-`--onchain-etherscan-api-key` 更适用于访问以太坊区块浏览器提供的服务，比如获取交易历史、智能合约信息等
- `ETH_RPC_URL` 则更适用于直接与以太坊节点进行交互，执行更高级的操作，如发送交易、查询区块链状态等。

### 获取存储
ItyFuzz 将从 Etherscan 拉取合同的 ABI 并对其进行模糊测试。 如果 ItyFuzz 在内存中遇到未知的插槽，则将从链 RPC 中拉取该插槽。 如果 ItyFuzz 遇到对外部未知合同的调用，则将获取该合同的字节码和 ABI。 如果其 ABI 不可用，则 ItyFuzz 将对其进行反编译并获取 ABI。
当遇到 SLOAD 与目标未初始化的时，ItyFuzz 尝试从区块链节点获取存储。有2种获取方式：
1）OneByOne：一次获取一个 slot 。这是默认模式。它很慢，但不会失败。
2）Dump：使用 debug API debug_storageRangeAt 来转储存储。这只适用于 ETH（目前），并且很容易失败。

### 测试时间—30min


### 构造函数参数
https://docs.ityfuzz.rs/docs-evm-contract/constructor-for-offchain-fuzzing
ItyFuzz 提供两种方法来传入构造函数参数。这些参数对于在部署时初始化合约的状态是必要的。
1）方法 1：CLI 参数
2）服务器转发