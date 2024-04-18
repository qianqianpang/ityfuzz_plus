use std::collections::HashMap;
use sentry::protocol::Addr;
use pyo3::prelude::*; // 导入 PyO3 的预定义符号
use pyo3::wrap_pyfunction; // 导入用于包装 Python 函数的宏
use pyo3::types::IntoPyDict; // 导入 IntoPyDict trait
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use libafl::inputs::Input;
use crate::evm::input::{EVMInput, EVMInputT, EVMInputTy};
use std::sync::RwLock;
use bytes::Bytes;
use revm_primitives::Env;
use crate::evm::mutator::AccessPattern;
use crate::evm::types::EVMAddress;
use crate::state_input::StagedVMState;


// mutate中传入的参数input是一个泛型参数，这里是EVMINPUT类型的
lazy_static! {
    pub static ref GLOBAL_INPUT: Mutex<EVMInput> = Mutex::new(EVMInput {
        input_type: EVMInputTy::ABI,
        caller: EVMAddress::default(),
        contract: EVMAddress::default(),
        data: None,
        sstate: StagedVMState::default(),
        sstate_idx: 0,
        txn_value: None,
        step: false,
        env: Env::default(),
        access_pattern: Arc::new(Mutex::new(AccessPattern::default())),
        liquidation_percent: 0,
        direct_data: Bytes::new(),
        randomness: Vec::new(),
        repeat: 0,
        swap_data: HashMap::new(),
    });
}

pub fn get_global_input() -> EVMInput {
    GLOBAL_INPUT.lock().unwrap().clone()
}

pub fn set_global_input(new_input: EVMInput) {
    // Lock the Mutex and set the new value
    *GLOBAL_INPUT.lock().unwrap() = new_input;
}

#[pymodule]
fn rust_python_dqn(_py: Python, m: &PyModule) -> PyResult<()> {
    // 将 Rust 函数包装成 Python 函数，并添加到模块中
    m.add_function(wrap_pyfunction!(dqn_algorithm, m)?)?;
    Ok(())
}

// 定义 Rust 函数，将其包装成 Python 函数
#[pyfunction]
fn dqn_algorithm() -> PyResult<()> {
    // 使用 Python::with_gil() 方法获取全局解释器锁 (GIL)，并在闭包中执行 Python 相关操作
    Python::with_gil(|py| {
        // 定义 Python 代码字符串，实现 DQN 算法的部分代码将放在这里
        let code = r#"

        "#;
        // 定义 Python 代码运行的作用域，导入 numpy 和 torch 模块
        let scope = [("numpy", py.import("numpy")?), ("torch", py.import("torch")?)].into_py_dict(py);
        // 使用 py.run() 方法执行 Python 代码，并传入作用域
        let _result = py.run(code, Some(&scope), None)?;
        Ok(()) // 返回 PyResult，表示操作成功
    })
}
