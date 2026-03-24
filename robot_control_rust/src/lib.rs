use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::slice;

// 示例：导出给 C/Python 调用的连接并测定接口
#[no_mangle]
pub extern "C" fn robot_ping(port_name: *const c_char, baud_rate: u32) -> c_int {
    if port_name.is_null() {
        return -1;
    }

    let c_str = unsafe { CStr::from_ptr(port_name) };
    let port_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    // 伪代码: 真实实现中这里应连接串口并发送 Ping，然后等待回复或返回状态
    // 此处仅验证跨语言 FFI 导出成功
    println!(
        "FFI Call Received: Ping port {} at {} baud",
        port_str, baud_rate
    );

    // 返回 0 表示成功, 这里立刻返回给外部调用者验证
    0
}

#[no_mangle]
pub extern "C" fn robot_get_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    let c_str = CString::new(version).unwrap();
    c_str.into_raw()
}

// 需要提供释放内存接口，避免内存泄漏
#[no_mangle]
pub unsafe extern "C" fn robot_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = CString::from_raw(s);
}

// 零拷贝协议解码范例函数包装 (通过 nom 解析器等)
#[no_mangle]
pub extern "C" fn robot_parse_fast(data: *const u8, len: usize) -> c_int {
    if data.is_null() || len == 0 {
        return -1;
    }

    let slice = unsafe { slice::from_raw_parts(data, len) };

    // 利用 nom 处理 slice...
    // 这里验证传入包长度，真实环境会对 slice 进行无分配解码
    if slice.len() > 1024 {
        1 // 返回状态示例
    } else {
        0
    }
}
