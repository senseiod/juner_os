#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(box_syntax)]
#![feature(wake_trait)]
extern crate alloc;
extern crate rlibc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;
use log::*;
use task::keyboard;
use task::{executor::Executor, Task}; // new

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod mal;
pub mod memory;
pub mod serial;
pub mod stdio;
pub mod task;
mod vga_buffer;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

/// 这个函数将在panic时被调用
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

// 函数入口
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use memory::BootInfoFrameAllocator;
    use x86_64::structures::paging::mapper::MapperAllSizes;
    use x86_64::structures::paging::Page;
    use x86_64::VirtAddr;
    // println!("Hello World {}", ",my friends!");
    init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // new: initialize a mapper
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    // init heap 初始化堆
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // 启动任务执行器
    let mut executor = Executor::new();
    executor.spawn(Task::new(mal::shell::mal_repl()));
    executor.run();
    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    // println!("init start");
    // 中断表初始化
    interrupts::init_idt();
    // 设置段表和 TSS
    gdt::init();
    // init log
    init_log();
    // PICS(中断控制器) 初始化
    unsafe { interrupts::PICS.lock().initialize() };
    // 允许时间中断
    x86_64::instructions::interrupts::enable();

    // println!("init end");
}

fn init_log() {
    struct SimpleLogger;
    impl Log for SimpleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            true
        }

        fn log(&self, record: &Record) {
            println!("[{:>5}] {}", record.level(), record.args());
        }

        fn flush(&self) {}
    }

    static LOGGER: SimpleLogger = SimpleLogger;
    set_logger(&LOGGER).unwrap();
    set_max_level(LevelFilter::Trace);
}

pub fn test() {
    use crate::mal::env::Env;
    use crate::mal::env::{env_new, env_sets};
    use crate::mal::rep;
    use crate::mal::types::format_error;
    use crate::mal::types::func; // 这个方法可以用 rust的闭包生成一个lisp的函数
    use crate::mal::types::MalArgs;

    // FIXME: 这里的注释识别有一些问题！！！ 不能正常的识别双引号
    let kernel_env: Env = env_new(None);
    use crate::mal::core::load_core;
    load_core(&kernel_env);

    let code = vec![
        // "(= 2 2)",
        // "(def! plus3 (lambda [x] (+ 3 x)))",
        // "(plus3 3)",
        // "(not false)",
        // "(do (+ 1 2) (* 3 3) 5)",
        // "\"Lisp is so
        // good  for me\"",
        // "(eval (read-string \"(+ 1 3)\"))",
        // "(def! test-eval (list + 3 3))",
        // "(eval test-eval)",
        // "(prn abc)",
        // "(prn (quote abc))",
        // "(prn 'abc)",
        // "(def! lst '(2 3))",
        // "(quasiquote (1 (unquote lst)))",
        // "(quasiquote (1 (splice-unquote lst)))",
        // "`(1 ~lst)",
        // "`(1 ~@lst)",
        // "(cons [1] [2 3])",
        // "(cons 1 [2 3])",
        // "(concat [1 2] (list 3 4) [5 6])",
        // "(concat [1 2])"
        // "(defmacro! unless (lambda (pred a b) `(if ~pred ~b ~a)))",
        // "(unless false 7 8)",
        // "(macroexpand (unless false 7 8))"
        // "(nth [1 2 3] 0)",
        // "(nth '(1 2 3) 1)",
        // "(first '((1 2) 2 3))",
        // "(count '(1 2 (2 3)))",
        // "(count [1 2 3])",
        // "(empty? '())",
        // "(empty? nil)",
        // "(throw \"err1\")",
        // "(try* abc (catch* exc (prn \"exc is:\" exc)))",
        // "(try* (throw \"my exception\") (catch* exc (do (prn \"exc:\" exc) 7)))",
        // "(apply + (list 1 3))",
        // "(apply + '(2 3))",
        // "(apply (lambda [x y] (do (prn  x \"+\" y) (+ x y))) '(7 8))",
        // "(map (lambda [x] (apply + x)) (list [1 2] [2 3]))",
        // "(def! *test-atom* (atom 0))",
        // "(reset! *test-atom* 10)",
        // "(deref *test-atom*)",
        // "(swap! *test-atom* (lambda [x] (+ x 1)))",
        // "@*test-atom*",
        // "(str \"sss\")",
        // "(def! test-hash (hash-map))",
        // "(assoc test-hash \"a\" 1)",
        // "(assoc test-hash \"b\" 2)",
        // "(get {\"a\" 1} \"a\")",
        // "(get {:a 10 :b {:c 3}} (keyword \"a\"))",
        // "(get {:a 10 :b {:c 3}} :b)",
        // "(keys {:a 1 :b 2 :c 3})",
        // "(not true)",
        "(gensym)",
        "(gensym)",
        // "((lambda (cont) (cont 2)) (lambda [x] (+ 1 x)))",
        // "((lambda [x] \"hi\") (lambda [x] \"hi\"))",
        // "
        //     (def! ten-test (lambda [data]
        //         (cond
        //             (> data 10) 1
        //             (= data 10) 0
        //             (< data 10) -1)))
        // ",
        // "(ten-test 15)",
        // "(def! call/cc (lambda))"
    ];

    for line in code {
        match rep(line, &kernel_env) {
            Ok(out) => println!("{}", out),
            Err(e) => println!("{}", format_error(e)),
        }
    }
}
