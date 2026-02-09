pub mod containers;

type Vec2D<T> = Vec<Vec<T>>;

#[cfg(test)]
mod tests {
    use super::*;

    mod imct {
        use super::*;
        use containers::imct::{ ColTable };
        use containers::traits::Table;

        #[derive(Clone, Default, Debug)]
        enum TestData {
            #[default] None,
            I32(i32),
            U8(u8),
            String(String)
        }

        #[test]
        fn basic() {
            let nrows: usize = 10;

            let table = ColTable::builder()
                .column("A", vec![TestData::I32(0); nrows])
                .column("B", vec![TestData::U8(0); nrows])
                .column("C", vec![TestData::String("Hello".to_string()); nrows])
                .build();

            let table_empty: ColTable<Vec2D<u8>> = ColTable::builder().build();

            assert_eq!(
                std::mem::size_of::<ColTable<Vec2D<TestData>>>(),
                std::mem::size_of::<ColTable<Vec2D<u8>>>()
            )
        }

        #[test]
        fn empty_table() {
            let table: ColTable<Vec2D<u8>> = ColTable::builder().build();
            assert_eq!(table.nrows(), 0);
            assert_eq!(table.ncols(), 0);
            assert!(table.is_empty());
        }

        #[test]
        fn temp() {
            // println!("{}", std::mem::size_of::<String>());
        }
    }
}
