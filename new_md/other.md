# 文件夹作用
### blaz
1. `src/evm/blaz/builder.rs`：这个文件定义了与构建作业相关的结构和方法。它包括提交作业、等待作业完成、处理作业结果等功能。这个文件中的代码主要用于与构建服务器交互，获取和处理构建结果。
2. `src/evm/blaz/offchain_config.rs`：这个文件定义了离链配置的结构和方法。它包括从 JSON URL、文件或字符串中获取配置信息。这个文件中的代码主要用于处理离链合约的配置信息。
3. `src/evm/blaz/offchain_artifacts.rs`：可能是处理离链工件（artifacts）的代码。在区块链中，工件通常指的是编译后的智能合约代码和相关信息。
4. `mod.rs`：这个文件包含了一些断言语句，这些语句用于测试 `is_bytecode_similar_lax` 函数的功能。这个文件可能是用于测试上述功能是否正常工作的。


### concolic
1. `src/evm/concolic/concolic_stage.rs`：这个文件定义了一个名为ConcolicStage的结构体，它是用于执行混合符号执行（Concolic Execution）的阶段。混合符号执行是一种程序分析技术，结合了符号执行和具体执行，用于自动发现程序错误或生成满足特定条件的输入。ConcolicStage包含了一些配置选项，如是否启用混合符号执行，是否允许符号地址等。它还包含了一些状态信息，如已知的状态输入，虚拟机执行器等。此外，这个文件还定义了一些与混合符号执行相关的元数据结构和方法。  
2. `src/evm/concolic/expr.rs`：这个文件定义了一些用于表示和处理混合符号执行中的表达式的结构和方法。这些表达式用于表示和处理程序中的符号变量和操作。例如，ConcolicOp枚举定义了各种可能的操作，如加法、除法、乘法、减法等。Expr结构体则用于表示一个表达式，它包含了左操作数、右操作数和操作。此外，这个文件还提供了一些用于操作和简化表达式的函数和宏。  
   BALANCE, CALLVALUE, CALLER, ORIGIN: 这些是EVM的特殊操作，分别表示获取账户余额、调用值、调用者、原始调用者。
3. `src/evm/concolic/concolic_host.rs`：这个文件定义了一个名为ConcolicHost的结构体，它是用于执行混合符号执行的主机。ConcolicHost包含了一些状态信息，如当前的调用上下文，当前的内存状态等。此外，这个文件还定义了一些与混合符号执行相关的方法，如在每一步执行时的操作，解决约束等。 


### middleware

1)call_printer:
CallType：这是一个枚举，定义了各种可能的调用类型，包括 Call、CallCode、DelegateCall、StaticCall、FirstLevelCall 和 Event。  
SingleCall：这是一个结构体，用于存储单个调用的信息，包括调用类型、调用者、合约、输入、值、源代码位置和结果。  
CallPrinterResult：这是一个结构体，用于存储所有调用的信息。它包含一个元组的向量，每个元组包含一个层级和一个 SingleCall。  
CallPrinter：这是主要的中间件结构体。它包含了一些字段，如地址到名称的映射、当前层级、结果、偏移量和一个表示是否是新交易的标志。它还定义了一些方法，如 new（创建新的 CallPrinter）、cleanup（清理当前的结果）、mark_new_tx（标记新的交易）、mark_step_tx（标记步骤交易）、get_trace（获取当前的调用跟踪）、save_trace（保存当前的调用跟踪）和 translate_address（将地址转换为名称）。  
Middleware 实现：这是 CallPrinter 对 Middleware 特性的实现。它定义了 on_step 和 on_return 方法，这两个方法在每个步骤和返回时被调用。在 on_step 方法中，它检查当前的指令，如果是一个外部调用或事件，它会记录相关的信息。在 on_return 方法中，它会更新结果，并减少当前的层级。它还定义了 get_type 方法，返回中间件的类型。

2）cheadcode
`Cheatcode<SC>`：这个结构体用于记录存储读写和日志。其中，`accesses`字段记录了存储的读写操作，`recorded_logs`字段记录了日志。`_phantom`字段是一个占位符，用于标记结构体中的泛型`SC`。
`Prank`：这个结构体用于处理“恶作剧”操作。在EVM中，每个交易都有一个`msg.sender`和`tx.origin`，分别表示消息的发送者和交易的原始发送者。通过“恶作剧”操作，可以临时改变这两个值。`old_caller`和`old_origin`字段分别记录了恶作剧开始时的`msg.sender`和`tx.origin`，`new_caller`和`new_origin`字段则是要设置的新值。`single_call`字段表示是否在下一次调用后自动停止恶作剧，`depth`字段记录了恶作剧开始时的调用深度。
`RecordAccess`：这个结构体用于记录存储槽的读写操作。`reads`字段记录了每个地址的读操作，`writes`字段记录了每个地址的写操作。
`ExpectedRevert`：这个结构体用于处理预期的`revert`操作。在EVM中，`revert`操作会停止当前的调用，并退回所有未使用的Gas。`reason`字段记录了预期的`revert`数据，`depth`字段记录了预期`revert`的深度。
`ExpectedEmit`：这个结构体用于记录预期的日志事件。在智能合约中，可以通过emit关键字来触发事件，并将事件记录在日志中。这个结构体包含了预期事件的一些信息，如预期的深度（depth）、预期的日志（log）、需要进行的检查（checks）、发起事件的地址（address）以及是否在子调用中找到了预期的日志（found）。  
`ExpectedCallData`：这个结构体用于记录预期的调用数据。在智能合约中，可以通过调用其他合约的函数来实现各种功能。这个结构体包含了预期调用的一些信息，如预期的调用值（value）、预期的调用次数（count）以及预期的调用类型（call_type）。  
`ExpectedCallType`：这个枚举定义了预期的调用类型，包括至少一次的调用（NonCount）和精确次数的调用（Count）。  
`OpcodeType`：这个枚举定义了操作码的类型，包括调用cheatcode地址（CheatCall）、调用其他地址（RealCall）、存储加载和存储（Storage）、撤销（Revert）、日志（Log）以及其他不关心的操作（Careless）。  
`try_or_continue和cheat_call_error`：这两个宏用于处理错误。try_or_continue宏用于尝试执行一个可能会失败的操作，如果操作失败，它会打印错误信息并立即返回。cheat_call_error宏用于处理调用cheatcode地址失败的情况，它会打印错误信息，并将指令结果设置为错误，然后推入一个零值到栈中，并将指令指针向前移动一位，最后立即返回。
`Middleware实现`：Cheatcode实现了Middleware特性，定义了on_step和get_type两个方法。on_step方法在每个步骤执行时被调用，它会根据当前的操作码类型执行不同的操作，如调用cheatcode地址、调用其他地址、记录存储访问和记录日志。get_type方法返回中间件的类型。  
`cheat_call方法`：这个方法用于处理调用cheatcode地址的情况。它首先从栈中弹出调用数据，然后尝试调整内存大小。接着，它会处理虚拟机调用，并根据调用的结果设置返回数据。最后，它会将指令指针向前移动一位，跳过当前的指令。  
`VmCalls处理`：在处理虚拟机调用时，它会根据调用的类型执行不同的操作。例如，warp操作用于设置环境的时间戳，roll操作用于设置环境的区块高度，fee操作用于设置环境的基础费用，load和store操作用于读取和写入存储，expectCall操作用于预期一个特定的调用等。


3)middleware.rs
`MiddlewareOp枚举`：这个枚举定义了一系列中间件操作。每个操作都与一个特定的中间件类型（MiddlewareType）相关联，并可能包含一些额外的参数。例如，UpdateSlot操作用于更新一个存储槽，它需要一个地址和两个U256值；AddCorpus操作用于添加一个语料库，它需要一个字符串和一个地址；Owed和Earned操作用于记录欠款和收入，它们需要一个U512值；MakeSubsequentCallSuccess操作用于使后续的调用成功，它需要一个字节序列。

4)reentrancy.rs
实现了一个检测重入攻击的中间件
`merge_sorted_vec_dedup`函数，该函数的主要目的是合并两个已排序的向量，并删除重复的元

5)sha3_bypass.rs
实现了一个用于绕过SHA3哈希函数的中间件。在某些情况下，我们可能希望绕过SHA3的计算，直接使用预先计算好的结果，这个中间件就是用来实现这个功能的。


6）coverage.rs
`instructions_pc`这个函数的作用是分析字节码，找出所有的指令和JUMPDEST的位置，以及需要跳过的位置。
返回一个元组，包含三个部分：  
指令PCs：这是一个集合，包含了所有指令的PCs//程序计数器（PC）是一个用于存储下一条指令的内存地址的寄存器。在这个上下文中，PCs是字节码中的偏移量，它们指向的是指令或者JUMPDEST。  
JUMPI PCs：这是一个集合，包含了所有JUMPI指令的PCs。JUMPI是以太坊虚拟机（EVM）的一个指令，用于进行条件跳转。  
Skip PCs：这是一个集合，包含了所有需要跳过的PCs。这些PCs可能是因为它们对应的指令是STOP或INVALID，或者是因为其他原因需要被跳过

### presets
`src/evm/presets/mod.rs` 中定义了 Preset 特质，以及一些与该特质相关的结构体和方法。Preset 特质定义了一个名为 presets 的方法，该方法接受一个函数签名、一个 EVMInput 输入和一个 EVMExecutor 执行器，返回一个 EVMInput 向量。此外，该文件还定义了 FunctionSig 结构体，用于表示函数签名，以及 ExploitTemplate 结构体，用于表示利用模板。ExploitTemplate 结构体包含一个从文件名创建实例的方法，该方法读取一个文件，将其内容解析为 ExploitTemplate 对象的向量。这些结构体和方法可能用于处理特定的以太坊虚拟机（EVM）操作，特别是与函数签名和利用模板相关的操作。
`src/evm/presets/pair.rs` 中定义了一个名为 PairPreset 的结构体，该结构体实现了 Preset 特质。在 PairPreset 结构体中，presets 方法的实现检查函数签名是否为 [0xbc, 0x25, 0xcf, 0x77]，如果是，则克隆输入，设置其 repeat 字段为 37，设置其 data 字段为一个包含地址的 BoxedABI 对象，并将新的输入添加到结果向量中。这个结构体和方法可能用于处理特定的以太坊虚拟机（EVM）操作。  


### producers

1)erc20.rs
定义了一个名为ERC20Producer的结构体，该结构体实现了Producer特质，用于处理以太坊虚拟机（EVM）中的ERC20代币相关操作。  
`ERC20Producer结构体包含两个字段：  `
balances：一个哈希映射，键是一个元组，包含一个调用者地址和一个代币地址，值是一个EVMU256类型的余额。这个字段用于存储每个调用者在每个代币合约中的余额。  
balance_of：一个字节向量，包含了ERC20代币合约的balanceOf函数的函数签名。这个字段用于在调用代币合约时指定要调用的函数。  
`ERC20Producer结构体实现了Producer特质，定义了两个方法：`  
produce方法：这个方法接受一个OracleCtx上下文对象的可变引用。它首先从上下文对象中获取一批代币地址和一批调用者地址，然后为每个调用者和每个代币创建一个调用数据，调用数据包含了balanceOf函数的函数签名和调用者的地址。然后，它会执行这些调用，并将每个调用的结果解析为一个余额，然后将这个余额存储到balances字段中。  
notify_end方法：这个方法接受一个OracleCtx上下文对象的可变引用。它会清空balances字段，以准备下一轮的操作。
2）pair.rs
定义了一个名为PairProducer的结构体，该结构体实现了Producer特质，用于处理以太坊虚拟机（EVM）中的配对操作。 
`PairProducer结构体包含两个字段：  `
reserves：一个哈希映射，键是一个EVMAddress类型的地址，值是一个元组，包含两个EVMU256类型的储备。这个字段用于存储每个地址对应的储备。  
fetch_reserve：一个字节向量，包含了获取储备的函数的函数签名。这个字段用于在调用合约时指定要调用的函数。  
`PairProducer结构体实现了Producer特质，定义了两个方法： ` 
produce方法：这个方法接受一个OracleCtx上下文对象的可变引用。它首先从上下文对象中获取一批地址，然后为每个地址创建一个调用数据，调用数据包含了获取储备的函数的函数签名和地址。然后，它会执行这些调用，并将每个调用的结果解析为一个储备，然后将这个储备存储到reserves字段中。  
notify_end方法：这个方法接受一个OracleCtx上下文对象的可变引用。它会清空reserves字段，以准备下一轮的操作。


# 关键变量
jmps: 这是一个指向JMP_MAP的可变引用，JMP_MAP可能是一个用于跟踪程序中所有跳转指令的映射。在这个映射中，键可能是跳转指令的地址，值可能是跳转的次数或其他相关信息。  
cmps: 这是一个指向CMP_MAP的可变引用，CMP_MAP可能是一个用于跟踪程序中所有比较指令的映射。在这个映射中，键可能是比较指令的地址，值可能是比较的次数或其他相关信息。  
reads: 这是一个指向READ_MAP的可变引用，READ_MAP可能是一个用于跟踪程序中所有读取操作的映射。在这个映射中，键可能是读取操作的地址，值可能是读取的次数或其他相关信息。  
writes: 这是一个指向WRITE_MAP的可变引用，WRITE_MAP可能是一个用于跟踪程序中所有写入操作的映射。在这个映射中，键可能是写入操作的地址，值可能是写入的次数或其他相关信息。  
jmp_observer: 这是一个StdMapObserver实例，它被用来观察jmps映射。StdMapObserver可能是一个用于观察和报告映射变化的工具。
deployer：部署者
evm_executor
fuzz_host