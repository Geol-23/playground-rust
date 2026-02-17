pub mod windows;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let size1 = windows::get_terminal_size().unwrap();
        let size2 = windows::get_terminal_size().unwrap();
        let _size3 = windows::get_terminal_size().unwrap();
        let _size4 = windows::get_terminal_size().unwrap();
        let _size5 = windows::get_terminal_size().unwrap();
        assert_eq!(size1.0, size2.0);
        assert_eq!(size1.1, size2.1);
        // println!("{:?}", size1);
        // println!("{:?}", size2);
        // println!("{}", "=".repeat(size1.0));
    }
}
