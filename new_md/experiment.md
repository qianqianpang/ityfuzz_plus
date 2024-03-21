### EVM 执行器: revm 

还利用 revm 的解释器钩子来执行动态仪表、收集dataflow和comparison信息，并进行快速快照


### 数据集：

ityfuzz
我们利用三个数据集（B1、B2 和 B3）来评估我们的工具。
B1（摘自 VERISMART [28]）包含 57 个支持 ERC20 标准接口（即代币）的智能合约。
B2 和 B3 分别包含 72 份和 500 份从以太坊链中抓取的智能合约。


### RPC 

Ityfuzz 将优先读取 ETH_RPC_URL 环境变量作为 RPC 地址，如果没有设置，将使用内置的公共 RPC 地址。
您可以通过提供地址，块和链来 fuzz 一个项目。
ityfuzz evm -o -t [TARGET_ADDR] --onchain-block-number [BLOCK] -c [CHAIN_TYPE] --onchain-etherscan-api-key [Etherscan API Key].RPC地址是什么

RPC地址（Remote Procedure Call Address）是一个网络地址，它用于指定在哪里可以通过远程过程调用（RPC）访问特定的服务或资源。在这个上下文中，RPC地址是一个URL，它指向一个运行Ethereum节点的服务器，这个节点提供了一个JSON-RPC接口，可以用来查询区块链的状态，发送交易，调用智能合约等。
例如，一个典型的Ethereum RPC地址可能看起来像这样：`http://localhost:8545`。这个地址指向运行在本地机器上的Ethereum节点，该节点监听8545端口上的JSON-RPC请求。
在ItyFuzz中，你可以通过设置`ETH_RPC_URL`环境变量来指定RPC地址。如果你没有设置这个环境变量，ItyFuzz将使用一个内置的公共RPC地址。
你也可以在命令行参数中指定RPC地址，例如：`ityfuzz evm -o -t [TARGET_ADDR] --onchain-block-number [BLOCK] -c [CHAIN_TYPE] --onchain-etherscan-api-key [Etherscan API Key] --rpc-url [RPC_URL]`。在这个例子中，`[RPC_URL]`应该被替换为你想要使用的RPC地址。