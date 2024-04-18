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
1. 超参数最佳？p：随机的概率；A,B,C,D, E：价值计算公式的超参数；x,y,z：更新PTable的超参数还没有加到函数
2. 能不能实现更多的变异算子。env_chain_id要不要加
3. 测试环境是啥？提前部署需要用到的？
4. should_havoc 还没考虑
5. 考虑突变叠加？多个突变阶段
6. 当有两个.solc文件时，编译哪个

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

### 已完成

1. GLOBAL_INPUT线程安全问题
修改：
pub access_pattern: Arc<Mutex<AccessPattern>>, &std::sync::Arc<Mutex<AccessPattern>>  Arc::new(Mutex::new(AccessPattern::new())),
pub trait ABI: CloneABI + Send +Sync
access_pattern: Arc::new(Mutex::new(AccessPattern::new()))

### todo
testcase  还是input作为state


