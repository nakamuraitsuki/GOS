use crate::println;
use crate::x86::hlt;

pub const SYSCALL_PRINT: u64 = 1;
pub const SYSCALL_EXIT: u64 = 60;

pub fn handle_syscall(num: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match num {
        // write(fd, buf, count)
        SYSCALL_PRINT => {
            if arg1 == 1 || arg1 == 2 { // stdout or stderr
                let ptr = arg2 as *const u8;
                let len = arg3 as usize;
                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                if let Ok(s) = core::str::from_utf8(slice) {
                    crate::print!("{}", s); // 改行を含んでいることが多いので print!
                }
                arg3 // 戻り値は「書き込んだバイト数」を返すのが一般的
            } else {
                0
            }
        }
        // exit(code)
        SYSCALL_EXIT => {
            println!("App exited with code: {}", arg1);
            loop { hlt(); } // 実際にはプロセスを終了させる必要があるが、一旦止める
        }
        // arch_prctl(code, addr)
        158 => {
            // Goはこれに成功しないとTLSが使えず、即座に死ぬ可能性が高いです。
            // 本来的にはFSレジスタのベースを設定する必要があります。
            println!("Warning: arch_prctl called. TLS base setting is ignored for now.");
            0 // 成功を偽装
        }
        // mmap
        9 => {
            println!("Warning: mmap called at {:#X} size {:#X}", arg1, arg2);
            // 本来はメモリを割り当てるべきですが、一旦固定のアドレスを返せれば動くかも？
            u64::MAX // 今はまだエラー
        }
        _ => {
            println!("Unknown syscall: {}", num);
            0 // 0を返して「成功したふり」をさせるほうが、先に進める場合があります
        }
    }
}
