use shadowjs_parser::Parser;
use shadowjs_bytecode::BytecodeCompiler;
use shadowjs_vm::VM;

pub struct ShadowEngine {
    vm: VM,
}

impl ShadowEngine {
    pub fn new() -> Self {
        Self { vm: VM::new() }
    }

    pub fn eval(&mut self, src: &str) -> Result<(), String> {
        let ast = Parser::new(src).parse()?;
        let bytecode = BytecodeCompiler::compile(&ast)?;
        self.vm.execute(bytecode)?;
        Ok(())
    }
}
