pub use alloc::vec::Vec;

#[macro_export]
macro_rules! vec {
    () => {{
        Vec::new()
    }};

    ($($sth:expr),+) => {{
        let mut temp_vec = Vec::new();

        $( temp_vec.push($sth); )+

        temp_vec
    }};

    ($init_value:expr ;  $size:expr) => {{
        let mut res = Vec::with_capacity($size);
        res.resize_with($size, || $init_value);
        res
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        let mut v = vec![1, 2, 3];
        v.extend_from_slice(&[4, 5, 6]);
    }
}
