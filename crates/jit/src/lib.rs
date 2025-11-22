use shadowjs_bytecode::{Chunk, Constant, OpCode};
use std::ptr;

#[cfg(windows)]
use windows_sys::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};

#[cfg(unix)]
use libc::{mmap, munmap, MAP_ANONYMOUS, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};

pub struct JitCompiler {
    // We keep track of allocated memory to free it later
    allocated_pages: Vec<(*mut u8, usize)>,
}

impl Drop for JitCompiler {
    fn drop(&mut self) {
        for (ptr, _size) in &self.allocated_pages {
            #[cfg(windows)]
            unsafe {
                VirtualFree(*ptr as *mut _, 0, MEM_RELEASE);
            }
            #[cfg(unix)]
            unsafe {
                munmap(*ptr as *mut _, *_size);
            }
        }
    }
}

impl JitCompiler {
    pub fn new() -> Self {
        Self {
            allocated_pages: Vec::new(),
        }
    }

    pub fn compile(&mut self, chunk: &Chunk) -> Result<fn() -> f64, String> {
        let mut assembler = Assembler::new();

        assembler.emit_push_rbp();
        assembler.emit_mov_rbp_rsp();

        for op in &chunk.code {
            match op {
                OpCode::Constant(idx) => {
                    match &chunk.constants[*idx] {
                        Constant::Number(n) => {
                            // Load constant to XMM0
                            assembler.emit_mov_xmm0_imm(*n);
                            // Push XMM0 to stack
                            assembler.emit_push_xmm0();
                        }
                        _ => return Err("JIT only supports numbers".to_string()),
                    }
                }
                OpCode::Add => {
                    // Pop b to XMM1
                    assembler.emit_pop_xmm1();
                    // Pop a to XMM0
                    assembler.emit_pop_xmm0();
                    // Add XMM1 to XMM0
                    assembler.emit_addsd_xmm0_xmm1();
                    // Push result
                    assembler.emit_push_xmm0();
                }
                OpCode::Sub => {
                    assembler.emit_pop_xmm1();
                    assembler.emit_pop_xmm0();
                    assembler.emit_subsd_xmm0_xmm1();
                    assembler.emit_push_xmm0();
                }
                OpCode::Mul => {
                    assembler.emit_pop_xmm1();
                    assembler.emit_pop_xmm0();
                    assembler.emit_mulsd_xmm0_xmm1();
                    assembler.emit_push_xmm0();
                }
                OpCode::Div => {
                    assembler.emit_pop_xmm1();
                    assembler.emit_pop_xmm0();
                    assembler.emit_divsd_xmm0_xmm1();
                    assembler.emit_push_xmm0();
                }
                OpCode::Return => {
                    // Pop result to XMM0 (return value)
                    // Check if stack is empty? For now assume one value left.
                    assembler.emit_pop_xmm0();

                    // Epilogue
                    // mov rsp, rbp
                    // pop rbp
                    // ret
                    assembler.emit_mov_rsp_rbp();
                    assembler.emit_pop_rbp();
                    assembler.emit_ret();
                }
                _ => return Err(format!("Unsupported opcode for SpiralX: {:?}", op)),
            }
        }

        let code = assembler.finalize();
        let ptr = self.allocate_executable_memory(&code)?;

        // Cast to function pointer
        let func = unsafe { std::mem::transmute::<_, fn() -> f64>(ptr) };

        Ok(func)
    }

    fn allocate_executable_memory(&mut self, code: &[u8]) -> Result<*const u8, String> {
        let size = code.len();

        #[cfg(windows)]
        unsafe {
            let ptr = VirtualAlloc(
                ptr::null_mut(),
                size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            );
            if ptr.is_null() {
                return Err("Failed to allocate memory".to_string());
            }
            ptr::copy_nonoverlapping(code.as_ptr(), ptr as *mut u8, size);
            self.allocated_pages.push((ptr as *mut u8, size));
            Ok(ptr as *const u8)
        }

        #[cfg(unix)]
        unsafe {
            let ptr = mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE | PROT_EXEC,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            );
            if ptr == libc::MAP_FAILED {
                return Err("Failed to allocate memory".to_string());
            }
            ptr::copy_nonoverlapping(code.as_ptr(), ptr as *mut u8, size);
            self.allocated_pages.push((ptr as *mut u8, size));
            Ok(ptr as *const u8)
        }
    }
}

struct Assembler {
    code: Vec<u8>,
}

impl Assembler {
    fn new() -> Self {
        Self { code: Vec::new() }
    }

    fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn emit_u64(&mut self, val: u64) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    fn emit_push_rbp(&mut self) {
        // 55
        self.emit_byte(0x55);
    }

    fn emit_mov_rbp_rsp(&mut self) {
        // 48 89 E5
        self.emit_bytes(&[0x48, 0x89, 0xE5]);
    }

    fn emit_mov_rsp_rbp(&mut self) {
        // 48 89 EC
        self.emit_bytes(&[0x48, 0x89, 0xEC]);
    }

    fn emit_pop_rbp(&mut self) {
        // 5D
        self.emit_byte(0x5D);
    }

    fn emit_ret(&mut self) {
        // C3
        self.emit_byte(0xC3);
    }

    fn emit_mov_xmm0_imm(&mut self, val: f64) {
        // mov rax, imm
        // 48 B8 imm64
        self.emit_bytes(&[0x48, 0xB8]);
        self.emit_u64(val.to_bits());
        // movq xmm0, rax
        // 66 48 0F 6E C0
        self.emit_bytes(&[0x66, 0x48, 0x0F, 0x6E, 0xC0]);
    }

    fn emit_push_xmm0(&mut self) {
        // sub rsp, 8
        // 48 83 EC 08
        self.emit_bytes(&[0x48, 0x83, 0xEC, 0x08]);
        // movsd [rsp], xmm0
        // F2 0F 11 04 24
        self.emit_bytes(&[0xF2, 0x0F, 0x11, 0x04, 0x24]);
    }

    fn emit_pop_xmm0(&mut self) {
        // movsd xmm0, [rsp]
        // F2 0F 10 04 24
        self.emit_bytes(&[0xF2, 0x0F, 0x10, 0x04, 0x24]);
        // add rsp, 8
        // 48 83 C4 08
        self.emit_bytes(&[0x48, 0x83, 0xC4, 0x08]);
    }

    fn emit_pop_xmm1(&mut self) {
        // movsd xmm1, [rsp]
        // F2 0F 10 0C 24
        self.emit_bytes(&[0xF2, 0x0F, 0x10, 0x0C, 0x24]);
        // add rsp, 8
        // 48 83 C4 08
        self.emit_bytes(&[0x48, 0x83, 0xC4, 0x08]);
    }

    fn emit_addsd_xmm0_xmm1(&mut self) {
        // addsd xmm0, xmm1
        // F2 0F 58 C1
        self.emit_bytes(&[0xF2, 0x0F, 0x58, 0xC1]);
    }

    fn emit_subsd_xmm0_xmm1(&mut self) {
        // subsd xmm0, xmm1
        // F2 0F 5C C1
        self.emit_bytes(&[0xF2, 0x0F, 0x5C, 0xC1]);
    }

    fn emit_mulsd_xmm0_xmm1(&mut self) {
        // mulsd xmm0, xmm1
        // F2 0F 59 C1
        self.emit_bytes(&[0xF2, 0x0F, 0x59, 0xC1]);
    }

    fn emit_divsd_xmm0_xmm1(&mut self) {
        // divsd xmm0, xmm1
        // F2 0F 5E C1
        self.emit_bytes(&[0xF2, 0x0F, 0x5E, 0xC1]);
    }

    fn finalize(self) -> Vec<u8> {
        self.code
    }
}
