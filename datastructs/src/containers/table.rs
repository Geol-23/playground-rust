pub trait Table {
    fn nrows(&self) -> usize;
    fn ncols(&self) -> usize;
    fn size(&self) -> (usize, usize) { (self.nrows(), self.ncols()) };
    fn is_empty(&self) -> bool { self.size() == (0, 0) }
    fn schema(&self) -> &dyn Schema;
}

pub trait Schema {
    type TypeId;

    fn field_count(&self) -> usize;
    fn field_name(&self, index: usize) -> &str;
    fn field_type(&self, index: usize) -> Self::TypeId;
}

pub trait TableMut: Table {

}

pub trait TableClone: Table {
    type Column;
    type Row;

    fn clone_column(&self, index: usize) -> Option<Self::Column>;
    fn clone_columns(&self, cols: std::ops::Range<usize>) -> Option<Vec<Self::Column>>;
    fn clone_row(&self, index: usize) -> Option<Self::Row>;
    fn clone_rows(&self, cols: std::ops::Range<usize>) -> Option<Vec<Self::Row>>;
}

pub trait TableTransform: Table {

}


pub mod view {
    use super::TableMut;

    // Read Only Views
    pub trait RowView: Table {
        type Row<'a>
        where
            Self: 'a;

        type RowIter<'a>: Iterator<Item = Self::Row<'a>>
        where
            Self: 'a;

        fn row(&self, index: usize) -> Option<Self::Row<'_>>;
        fn rows(&self) -> Self::RowIter<'_>;
    }

    pub trait ColumnView: Table {
        type Column<'a>
        where
            Self: 'a;

        type ColumnIter<'a>: Iterator<Item = Self::Column<'a>>
        where
            Self: 'a;

        fn column(&self, index: usize) -> Option<Self::Column<'_>>;
        fn columns(&self) -> Self::ColumnIter<'_>;
    }

    pub trait CellView: Table {
        type Cell<'a>
        where
            Self: 'a;

        fn cell(&self, r: usize, c: usize) -> &Self::Cell<'_>;
    }

    pub trait RangeView: Table {
        type Range<'a>
        where
            Self: 'a

        fn range(&self, rows: std::ops::Range<usize>, cols: std::ops::Range<usize>) -> Self::Range<'_>;
    }

    pub trait Locator<'a> {
        type Output;

        fn row(self, range: Range<usize>) -> Self;
        fn col(self, range: Range<usize>) -> Self;
        fn build(self) -> Self::Output;
    }

    pub trait TableView: RowView + ColumnView + CellView + RangeView {}


    // Mutable Views
    pub trait RowViewMut: TableMut {
        type RowMut<'a>
        where
            Self: 'a;

        type RowIterMut<'a>: Iterator<Item = Self::RowMut<'a>>
        where
            Self: 'a;

        fn row(&mut self, index: usize) -> Option<Self::RowMut<'_>>;
        fn rows(&mut self) -> Self::RowIterMut<'_>;
    }

}








pub mod sort {
    pub trait RowSortMut: TableMut + RowView {
        fn sort_rows_mut<K, F>(&mut self, f: F) -> Result<(), &'static str> {
            where
                F: Fn(&Self::Row<'_>) -> K,
                K: Ord
        }
    }

    pub trait RowSortClone: TableClone + RowView + Sized {
        fn sort_rows_clone<K, F>(&self, f: F) -> Result<Self, &'static str> {
            where
                F: Fn(&Self::Row<'_>) -> K,
                K: Ord
        }
    }

    pub trait RowSortTransform: TableTransform + RowView + Sized {
        fn sort_rows<K, F>(self, f: F) -> Result<Self, &'static str> {
            where
                F: Fn(&Self::Row<'_>) -> K,
                K: Ord
        }
    }



    pub trait ColumnSortMut: TableMut + ColumnView {
        fn sort_cols_mut<K, F>(&mut self, f: F) -> Result<(), &'static str> {
            where
                F: Fn(&Self::Column<'_>) -> K,
                K: Ord
        }
    }

    pub trait ColumnSortClone: TableClone + ColumnView + Sized {
        fn sort_cols_clone<K, F>(&self, f: F) -> Result<Self, &'static str> {
            where
                F: Fn(&Self::Column<'_>) -> K,
                K: Ord
        }
    }

    pub trait ColumnSortTransform: TableTransform + ColumnView + Sized {
        fn sort_cols<K, F>(self, f: F) -> Result<Self, &'static str> {
            where
                F: Fn(&Self::Column<'_>) -> K,
                K: Ord
        }
    }
}


pub mod map {
    pub trait MapMut: TableMut + RowView {

    }
}
