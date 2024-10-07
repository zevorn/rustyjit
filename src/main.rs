use std::mem;
use std::ops::{Index, IndexMut};

// 定义页面大小常量
const PAGE_SIZE: usize = 4096;

/// JIT内存管理结构
struct JitMemory {
    contents: *mut u8,
}

impl JitMemory {
    /// 创建新的 JitMemory 实例
    /// 
    /// # 参数
    /// - `num_pages`: 需要分配的页面数量
    /// 
    /// # 返回
    /// - `JitMemory`: 新创建的 JitMemory 实例
    fn new(num_pages: usize) -> JitMemory {
        let contents: *mut u8;
        unsafe {
            let size: usize = num_pages * PAGE_SIZE;
            let mut _contents: *mut libc::c_void =
                mem::MaybeUninit::<libc::c_void>::uninit().as_mut_ptr();
            // 分配内存并对齐到页面大小
            libc::posix_memalign(&mut _contents, PAGE_SIZE, size);
            // 设置内存权限为可执行、可读、可写
            libc::mprotect(
                _contents,
                size,
                libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE,
            );

            // 初始化内存内容为 'RET' 指令
            libc::memset(_contents, 0xc3, size);

            contents = mem::transmute(_contents);
        }

        JitMemory { contents: contents }
    }
}

// 实现 Index trait，允许通过索引访问内存
impl Index<usize> for JitMemory {
    type Output = u8; // 声明了 Index<usize> trait 的关联类型 Output 为 u8 类型

    fn index(&self, _index: usize) -> &u8 {
        unsafe { &*self.contents.offset(_index as isize) }
    }
}

// 实现 IndexMut trait，允许通过索引修改内存
impl IndexMut<usize> for JitMemory {
    fn index_mut(&mut self, _index: usize) -> &mut u8 {
        unsafe { &mut *self.contents.offset(_index as isize) }
    }
}

/// 编译并返回一个函数，该函数在 JIT 内存中生成指定的机器码
/// 
/// # 返回
/// - `fn() -> i64`: 一个在 JIT 内存中生成的函数，返回一个 i64 值
fn run_jit() -> fn() -> i64 {
    let mut jit: JitMemory = JitMemory::new(1);

    // 在 JIT 内存中写入机器码，生成一个返回 3 的函数
    jit[0] = 0x48; // mov RAX, 0x30
    jit[1] = 0xc7;
    jit[2] = 0xc0;
    jit[3] = 0x30;
    jit[4] = 0;
    jit[5] = 0;
    jit[6] = 0;

    // 将内存指针转换为函数指针并返回
    unsafe { mem::transmute(jit.contents) }
}

fn main() {
    // 生成并调用 JIT 编译的函数，并打印结果
    let fun: fn() -> i64 = run_jit();
    println!("{:#x}", fun());
}
