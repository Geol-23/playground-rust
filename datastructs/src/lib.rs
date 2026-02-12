pub mod containers;


#[cfg(test)]
mod tests {
    use super::*;

    mod imct {
        use super::*;
        use containers::table::{
            imct::{ ColTable},
            traits::{
                Table,
                loc::RowLocView
            }
        };

        fn create_test_table() -> ColTable {
            let nrows: usize = 1_000;

            ColTable::builder()
                .column("A", vec![0i32; nrows])
                .column("B", vec![0u8; nrows])
                .column("C", vec!["Hello".to_string(); nrows])
                .build()
        }

        #[test]
        fn basic() {
            let _ = create_test_table();
        }

        #[test]
        fn empty_table() {
            let table = ColTable::builder().build();
            assert_eq!(table.nrows(), 0);
            assert_eq!(table.ncols(), 0);
            assert!(table.is_empty());
        }

        #[test]
        fn rowlocview() {
            let table = create_test_table();
            assert_eq!(table.row(0).unwrap().get::<i32>(0).unwrap(), &0);

            let s = "Hello".to_string();
            for row in &table {
                let some_string: &String = row.get(2).unwrap();
                assert_eq!(some_string, &s);
            }
        }

        #[test]
        fn temp() {
            // println!("{}", std::mem::size_of::<String>());
        }
    }
}
