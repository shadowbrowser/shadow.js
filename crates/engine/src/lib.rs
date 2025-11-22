use shadowjs_bytecode::BytecodeCompiler;
use shadowjs_parser::Parser;
use shadowjs_vm::VM;

pub struct ShadowEngine {
    vm: VM,
}

impl ShadowEngine {
    pub fn new() -> Self {
        Self { vm: VM::new() }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.vm.set_debug(debug);
    }

    pub fn eval(&mut self, src: &str) -> Result<(), String> {
        let ast = Parser::new(src).parse()?;
        let bytecode = BytecodeCompiler::compile(&ast)?;
        self.vm.execute(bytecode)?;
        Ok(())
    }
}
