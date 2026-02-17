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
            let nrows: usize = 10;

            ColTable::builder()
                .column("A1", vec![usize::MAX; nrows])
                .column("A2", vec![usize::MAX; nrows])
                .column("A3", vec![usize::MAX; nrows])
                .column("A4", vec![usize::MAX; nrows])
                .column("A5", vec![usize::MAX; nrows])
                .column("A6", vec![usize::MAX; nrows])
                .column("A7", vec![usize::MAX; nrows])
                .column("Col B", vec![u8::MAX; nrows])
                .column("Column C", vec!["Hello".to_string(); nrows])
                .column("Column D", vec!["Hello".to_string(); nrows])
                .column("Column E", vec!["Hello".to_string(); nrows])
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
            let nrows: usize = 10;
            let table = ColTable::builder()
                .column("A", vec![i32::MAX; nrows])
                .column("Col B", vec![u8::MAX; nrows])
                .column("Column C", vec!["Hello".to_string(); nrows])
                .build();

            assert_eq!(table.row(0).unwrap().get::<i32>(0).unwrap(), &i32::MAX);

            let s = "Hello".to_string();
            for row in &table {
                let some_string: &String = row.get(2).unwrap();
                assert_eq!(some_string, &s);
            }
        }

        #[test]
        fn temp() {
            let table = create_test_table();
            println!("{}", table);
        }
    }
}
