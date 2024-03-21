### - 初始化变异器列表mutator_list：
    - 现有的变异器是基于二进制编译的,共22种
     mutator_list = [BitFlipMutator, ByteFlipMutator, ByteIncMutator, ByteDecMutator, ByteNegMutator, ByteRandMutator, ByteAddMutator, WordAddMutator, DwordAddMutator, QwordAddMutator, ByteInterestingMutator, WordInterestingMutator, DwordInterestingMutator, BytesSetMutator, BytesRandSetMutator, BytesSwapMutator, ConstantHintedMutator, VMStateHintedMutator, BytesExpandMutator, BytesInsertMutator, BytesRandInsertMutator, BytesCopyMutator]

    - 包括三个层面的变异器
      1 合约函数的参数
      2 tx
      3 环境参数



### - 初始化对应的概率表PTable
    随机生成概率值 or 均等概率值


### - 循环以下步骤
    结束条件：
    - 到达迭代次数
    - 找到bug

### 根据PTable选择一个变异器(action)，对已选择的种子进行变异、执行，收集反馈信息
    - 选择变异器：大部分时候直接选取最大概率的mutator，同时保留一定的随机性，一定概率p下进行随机选择（ p值得选取先大后小）
    - 选择种子：源代码已有选择机制
    - 变异：
    - 执行
    - 收集反馈信息
      Is_triggered：{0,1}
      Cov:{有符号double值}
      Is_datawaypoints_interesting：{0，1}
      Is_comparisonwaypoint_interesting：{0，1}
      Is_instructions_interesting：{0，1}



### 计算价值
    - 价值计算公式：
      value = A*Is_triggered + B*Cov_diff + C*Is_datawaypoints_interesting + D*Is_comparisonwaypoint_interesting + E*Is_instructions_interesting
      [(1,1,1,1,1),(3,2,1,1,1),(5,3,2,2,1)]

    - 更新PTable
      
### 更新PTable
    - y>VALUE>x，增加概率10%
    - 0<VALUE<x，增加概率5%
    - z<VALUE<0，不变
    - VALUE<z，减少概率5%



### 超参数：

p：随机选择变异器的概率
A,B,C,D, E：价值计算公式的超参数
x,y,z：更新PTable的超参数

Cov_diff