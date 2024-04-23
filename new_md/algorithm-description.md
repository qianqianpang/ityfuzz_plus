### Multi-armed bandit algorithm(power_sched.rs选择如何调整ptable)
#### 1.定义并补充变异操作：包括三个层面
      1 函数参数
      2 tx顺序==state 快照
      3 环境参数：
        -包括：调用者（caller）、余额（balance）、交易值（txn_value）、gas价格（gas_price）、基础费用（basefee）、时间戳（timestamp）、coinbase、gas限制（gas_limit）、区块号（number）和prevrandao。 ）
        -其他：gas_price 越高，交易越快得到处理。。貌似在离线测试没有影响gas_price.是由两部分组成的：基本费用（由协议自动计算）和优先费用（由用户输入）。基本费用根据网络拥堵情况而变化，优先费用由用户决定）
#### 2.定义变异概率表PTable（evm_fuzzer.rs中选择是否重置ptable）
    1) 自定义值
    2) 均等概率值
    3) 随机生成概率值

#### 3.反馈信息
    Is_triggered：{0,1}
    Cov:{有符号double值}，项目中有instruction_coverage和branch_coverage,但是项目有计算bug
    Is_datawaypoints_interesting：{0，1}
    Is_comparisonwaypoint_interesting：{0，1}
    Is_instructions_interesting：{0，1}//关键指令如状态变量的写入（SSTORE）、外部调用（CALL、CALLCODE、DELEGATECALL）和自毁（SELFDESTRUCT），就认为是有价值的

#### 4.循环以下步骤
    结束条件：到达迭代次数 or 找到bug
    1.选择变异器：大部分时候直接选取最大概率的mutator，同时保留一定的随机性，一定概率p下进行随机选择（ p值得选取先大后小）
    2.选择种子：源代码已有选择机制
    3.变异：
    4.执行
    5.收集反馈信息
    6.计算价值并更新PTable

##### 1)价值计算公式：
      value = A*Is_triggered + B*Cov_diff + C*Is_datawaypoints_interesting + D*Is_comparisonwaypoint_interesting + E*Is_instructions_interesting
      [(1,1,1,1,1),(3,2,1,1,1),(5,3,2,2,1)]
##### 2)PTable更新
    - y>VALUE>x，增加概率10%
    - 0<VALUE<x，增加概率5%
    - z<VALUE<0，不变
    - VALUE<z，减少概率5%
##### 3)超参数：
p：随机的概率
A,B,C,D, E：价值计算公式的超参数
x,y,z：更新PTable的超参数


### 已完成
1. global_info.rs  cheatcode.rs——增加指令有趣的反馈
2. 实现3个没有实现的env mutate：balance prevrandao gas_price:变异机制暂定，原值*（0-2）
   增加difficulty mutate
3. abi.rs/mutate_with_vm_slots函数 ——修改变异的规则，如不用确定的10%
4. input.rs/mutate_env_with_access_pattern函数——修改为不随机选择
5. input.rs/mutate函数 ——不再使用随机数控制变异(如 state.rand_mut())
6. 重写库函数中的 self.schedule(state, input)函数 ——不随机选择
   mutation_utils.rs    byte_mutator byte_mutator_expansion调用上面的
7. mutator.rs/mutate函数 ——不再使用随机数控制变异，而是使用模拟退火算法选择变异器进行变异（如具体的数值100 80 ；state.rand_mut()）
8. 计算value并更新ptable

### todo
##### **追踪github issue**
1. 超参数最佳？p：随机的概率；A,B,C,D, E：价值计算公式的超参数；x,y,z：更新PTable的超参数还没有加到函数
2. 能不能实现更多的变异算子。env_chain_id要不要加
3. 测试环境是啥？提前部署需要用到的？
4. should_havoc 还没考虑
5. 清算10  or 0
6. 考虑突变叠加？多个突变阶段
7. 当有两个.solc文件时，编译哪个

### =========================================

### 状态行动价值——DQN
- 目标gole：用时更短，找到bug，最大化覆盖率，（间接任务 使种子更有趣）
- 环境enveriment：当前测试的智能合约
- 其他：
   - Reward shaping技术可以让RL算法收敛得更快些
   - 感觉没必要设计惩罚项，来避免鲁莽，贪婪？不鲁莽之后避免胆怯？
      - 设置计数器，如果次数和他的贡献没有呈现正相关，就惩罚它
- gogal：学习好的变异器选择策略，让cov在内的reward最大化。本质上，DRL使得估计的reward和真实的reward更加接近



- 动作空间action：选择的变异操作
- 状态空间state：当前的测试用例或者加工后的表示（AST、embeding）；当前轮数，
- 回报函数reward：
    - 根据反馈信息 Is_triggered、Cov不能是差值、Is_datawaypoints_interesting、Is_comparisonwaypoint_interesting、Is_instructions_interesting
状态表示


定义一个神经网络模型，它将状态作为输入，输出每个可能动作的预期奖励。
初始化一个经验回放内存。它将存储过去的转换（状态，动作，奖励，新状态）。
定义损失函数
选择优化器
对于每个训练步骤：
    选择一个动作。有一定的概率随机选择一个动作（为了探索），否则选择模型预测的奖励最高的动作（为了利用）。
    执行选择的动作，获取新的状态和奖励，将转换（状态，动作，奖励，新状态）存储在经验回放内存中。
    从经验回放内存中随机抽取一批转换，用这些转换来更新模型。计算模型预测的Q值和实际的Q值（奖励加上新状态的最大预期奖励），然后根据这两者之间的差异来更新模型。


#### 设计
1. state_dim:12
    * caller: {revm_primitives::bits::B160}// 调用者地址
    * data: {core::option::Option<ityfuzz::evm::abi::BoxedABl>}
    * sstate:  {ityfuz.state input:stagedVMState<revm primitives:bits.:.160,revm primitives:bis:B160,ityfuzz.evm.vmEVMState,ityfuzevm.input:.ConciseEVMinput>} // 阶段性的VM状态 ,
    * txn_value: fcore::option::Option<ruint::Uint<...>>} // 交易值
    * step: {bool} // 是否从上一次控制泄漏处恢复执行
    * env: {revm_primitives::env::Env}// 环境（区块、时间戳等）,
    * access_pattern:  {alloc::rc::Rc<core::cell::RefCell..>, alloc::alloc::Global>}  // 访问模式
    * liquidation_percent: {u8} // 清算的代币百分比
    * direct_data: {bytes::bytes::Bytes} // 直接数据，即原始输入数据
    * randomness: {falloc::vec::Vec<u8,alloc::alloc::Global>} // 为突变器提供的额外随机字节
    * repeat: {u64} // 执行交易的次数
    * swap_data:  {std.collections:hash.map:HashMap<aloc..tring:.tring, ityfuz.generic_vm.m stateswapinfo, std.colections:hash.map.Randomstate> }// 交换数据
2. action_dim
   Key: "BYTE_MUTATIONS", Inner keys count: 16//Key: "BYTE_MUTATIONS_EXPANSION", Inner keys count: 19
   Key: "ENV", Inner keys count: 11
   Key: "TARRAY_DYNAMIC", Inner keys count: 3
   Key: "MUTATE_ALL", Inner keys count: 3
   Key: "INPUT_MUTATE", Inner keys count: 2
   Key: "T256_ADDRESS", Inner keys count: 2
   Key: "TUNKNOWN", Inner keys count: 2
   Key: "MUTATE_TEMPLATE", Inner keys count: 2
   Key: "MUTATE_STATE", Inner keys count: 2
   Key: "MUTATE_DATA", Inner keys count: 2
   Key: "MUTATE_BYTE", Inner keys count: 2
   Key: "MUTATE_BORROW", Inner keys count: 2
3. Adam优化器是一种用于深度学习模型的优化算法。它结合了两种扩展的随机梯度下降方法：自适应梯度算法（AdaGrad）和均方根传播（RMSProp）。Adam优化器计算自适应学习率，这意味着它保持每个参数的单独学习率，这些学习率根据参数的一阶矩估计（均值）和二阶矩估计（未中心的方差）进行调整。
除了Adam优化器，还有许多其他的优化器可供选择，包括：
    SGD（随机梯度下降）：这是最基本的优化器，但在某些情况下，它可能无法很好地收敛。
    Momentum：这是SGD的一个变种，它在更新中考虑了过去的梯度，以加速SGD在相关方向上的收敛速度。
    RMSprop：这是另一个可以加速SGD的优化器，它通过调整每个参数的学习率来实现。
    Adagrad：这个优化器在训练过程中调整学习率，对于稀疏数据集来说，它表现得很好。
    Adadelta：这是Adagrad的一个扩展，它试图减少Adagrad方法的急剧减小学习率的问题。
    Adamax：这是Adam的一个变种，它的稳定性更好。
    Nadam：这是Adam的一个变种，它结合了Adam和Nesterov的优点。
4. 损失函数，使用的是均方误差损失（Mean Squared Error Loss）
   `tch-rs`库实现了多种损失函数，包括但不限于以下几种：
    `mse_loss`: 均方误差损失函数（Mean Squared Error Loss）。
    `l1_loss`: L1损失函数，也称为绝对值损失函数。
    `cross_entropy_loss`: 交叉熵损失函数，常用于分类问题。
    `nll_loss`: 负对数似然损失函数（Negative Log Likelihood Loss），常用于多分类问题。
    `binary_cross_entropy`: 二元交叉熵损失函数，常用于二分类问题。
    `binary_cross_entropy_with_logits`: 带有logits的二元交叉熵损失函数，常用于二分类问题。
    `poisson_nll_loss`: 泊松负对数似然损失函数。
    `cosine_embedding_loss`: 余弦嵌入损失函数。
    `hinge_embedding_loss`: Hinge嵌入损失函数。
    `kl_div`: Kullback-Leibler散度损失函数。
这些损失函数可以用于不同的机器学习任务，包括回归、分类、序列预测等。你可以根据你的任务需求选择合适的损失函数。
5. 在`train`函数中：
对于每个训练周期（episode）：
    - 重置环境并获取初始状态
    - 在环境结束之前，不断执行以下步骤：
        - 对于每个动作维度，获取当前状态下的动作
        - 执行动作并获取下一个状态、奖励和是否结束
        - 将经验（当前状态、动作、奖励、下一个状态）存入回放缓冲区
        - 更新当前状态为下一个状态
        - 更新模型

在`evaluate`函数中：
对于每个评估周期（episode）：
    - 重置环境并获取初始状态
    - 在环境结束之前，不断执行以下步骤：
        - 对于每个动作维度，获取当前状态下的动作
        - 执行动作并获取下一个状态、奖励和是否结束
        - 更新当前状态为下一个状态
        - 累计奖励
    - 计算平均奖励

以下是根据这些步骤完成的`train`和`evaluate`函数：

#### 已完成

1. GLOBAL_INPUT线程安全问题
修改：
    pub access_pattern: Arc<Mutex<AccessPattern>>, &std::sync::Arc<Mutex<AccessPattern>>  Arc::new(Mutex::new(AccessPattern::new())),
    pub trait ABI: CloneABI + Send +Sync
    access_pattern: Arc::new(Mutex::new(AccessPattern::new()))
mutator。rs调用set GLOBAL_INPUT
2. state 设计，暂定4个
3. action编码  先实现全编码，后面再考虑如何剪枝/分层/只考虑主干啥的
4. 缺失依赖问题，利用Dependency Walker 工具来分析生成的可执行文件（.exe 文件），
                把libtorch的path放在LLVM前面
#### todo
1. env 的获取
   根据gobal_input依次获取state中的字段
2. 开始训练使用，将输出的action对接到代码调用
3. dqnnet网络结构调整





### ==================================================

### 其他

为什么不选择分层DQN，而是用端到端的全编码