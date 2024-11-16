use libc::c_char;

/// # Safety
///
/// If str is not a null terminated ascii string, this function will result in undefined behavior.
pub unsafe fn cstr_to_string(cstr_in: *const c_char) -> String {
    let mut index = 0;
    let mut str_out = String::new();
    loop {
        let curr_char: c_char = bytemuck::cast(*cstr_in.add(index));
        if curr_char.eq(&bytemuck::cast(b'\0')) {
            break;
        };
        str_out.push(curr_char as u8 as char);
        index += 1;
    }
    str_out
}

#[cfg(test)]
mod test {
    use std::ffi::CStr;

    use bytemuck::cast;
    use libc::{c_char, malloc};

    use crate::cstr_to_string;

    #[test]
    fn test_string_conversion() {
        let hello_rust_string = unsafe {
            let hello = malloc(size_of::<c_char>() * 6) as *mut c_char;
            *hello.add(0) = bytemuck::cast(b'h');
            *hello.add(1) = bytemuck::cast(b'e');
            *hello.add(2) = bytemuck::cast(b'l');
            *hello.add(3) = bytemuck::cast(b'l');
            *hello.add(4) = bytemuck::cast(b'o');
            *hello.add(5) = bytemuck::cast(b'\0');
            //cstr_to_string(hello)
            CStr::from_ptr(hello)
        };

        assert_eq!(hello_rust_string.to_str().unwrap(), "hello");
        let back_to_cstr = hello_rust_string.as_ptr();
        assert_eq!(unsafe { *back_to_cstr }, cast(b'h'));
        assert_eq!(unsafe { *back_to_cstr.add(1) }, cast(b'e'));
    }
}
