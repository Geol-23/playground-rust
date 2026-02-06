pub trait Table {
    type TypeId;

    fn nrows(&self) -> usize;
    fn ncols(&self) -> usize;
    fn size(&self) -> (usize, usize) { (self.nrows(), self.ncols()) }
    fn is_empty(&self) -> bool { self.size() == (0, 0) }
    fn schema(&self) -> &dyn Schema<TypeId = Self::TypeId>;
}

pub trait Schema {
    type TypeId;

    fn field_count(&self) -> usize;
    fn field_name(&self, index: usize) -> &str;
    fn field_type(&self, index: usize) -> Self::TypeId;
}

pub trait TableMut: Table { }

pub trait TableClone: Table {
    type Column;
    type Row;

    fn clone_column(&self, index: usize) -> Option<Self::Column>;
    fn clone_columns(&self, cols: std::ops::Range<usize>) -> Option<Vec<Self::Column>>;
    fn clone_row(&self, index: usize) -> Option<Self::Row>;
    fn clone_rows(&self, cols: std::ops::Range<usize>) -> Option<Vec<Self::Row>>;
}

pub trait TableTransform: Table { }


pub mod loc {
    use super::{ Table, TableMut, TableClone, TableTransform };

    // View Locators
    pub trait RowLocView: Table {
        type Row<'a>
        where
            Self: 'a;

        type RowIter<'a>: Iterator<Item = Self::Row<'a>>
        where
            Self: 'a;

        fn row(&self, index: usize) -> Option<Self::Row<'_>>;
        fn rows(&self) -> Self::RowIter<'_>;
    }

    pub trait ColumnLocView: Table {
        type Column<'a>
        where
            Self: 'a;

        type ColumnIter<'a>: Iterator<Item = Self::Column<'a>>
        where
            Self: 'a;

        fn col(&self, index: usize) -> Option<Self::Column<'_>>;
        fn cols(&self) -> Self::ColumnIter<'_>;
    }

    pub trait CellLocView: Table {
        type Cell<'a>
        where
            Self: 'a;

        fn cell(&self, r: usize, c: usize) -> &Self::Cell<'_>;
    }

    pub trait RangeLocView: Table {
        type Range<'a>
        where
            Self: 'a;

        fn range(&self, rows: std::ops::Range<usize>, cols: std::ops::Range<usize>) -> Self::Range<'_>;
    }

    pub trait TableLocView: RowLocView + ColumnLocView + CellLocView + RangeLocView {}



    // Mutable Locators
    pub trait RowLocMut: TableMut {
        type RowMut<'a>
        where
            Self: 'a;

        type RowIterMut<'a>: Iterator<Item = Self::RowMut<'a>>
        where
            Self: 'a;

        fn row_mut(&mut self, index: usize) -> Option<Self::RowMut<'_>>;
        fn rows_mut(&mut self) -> Self::RowIterMut<'_>;
    }

    pub trait ColumnLocMut: TableMut {
        type ColumnMut<'a>
        where
            Self: 'a;

        type ColumnIterMut<'a>: Iterator<Item = Self::ColumnMut<'a>>
        where
            Self: 'a;

        fn col_mut(&mut self, index: usize) -> Option<Self::ColumnMut<'_>>;
        fn cols_mut(&mut self) -> Self::ColumnIterMut<'_>;
    }

    pub trait CellLocMut: TableMut {
        type CellMut<'a>
        where
            Self: 'a;

        fn cell_mut(&mut self, r: usize, c: usize) -> &Self::CellMut<'_>;
    }

    pub trait RangeLocMut: TableMut {
        type RangeMut<'a>
        where
            Self: 'a;

        fn range_mut(&self, rows: std::ops::Range<usize>, cols: std::ops::Range<usize>) -> Self::RangeMut<'_>;
    }

    pub trait TableLocMut: RowLocMut + ColumnLocMut + CellLocMut + RangeLocMut {}


    // Clone Locators
    pub trait RowLocClone: TableClone {
        type RowClone;
        type RowIterClone: Iterator<Item = Self::RowClone>;

        fn row_clone(&self, index: usize) -> Option<Self::RowClone>;
        fn rows_clone(&self) -> Self::RowIterClone;
    }

    pub trait ColumnLocClone: TableClone {
        type ColumnClone;
        type ColumnIterClone: Iterator<Item = Self::ColumnClone>;

        fn col_clone(&self, index: usize) -> Option<Self::ColumnClone>;
        fn cols_clone(&self) -> Self::ColumnIterClone;
    }

    pub trait CellLocClone: TableClone {
        type CellClone;

        fn cell_clone(&self, r: usize, c: usize) -> Self::CellClone;
    }

    pub trait RangeLocClone: TableClone {
        type RangeClone;

        fn range_clone(&self, rows: std::ops::Range<usize>, cols: std::ops::Range<usize>) -> Self::RangeClone;
    }

    pub trait TableLocClone: RowLocClone + ColumnLocClone + CellLocClone + RangeLocClone {}


    // Transform Locators
    pub trait RowLocTransform: TableTransform {
        type RowTransform;
        type RowIterTransform: Iterator<Item = Self::RowTransform>;

        fn to_row(self, index: usize) -> Option<Self::RowTransform>;
        fn to_rows(self) -> Self::RowIterTransform;
    }

    pub trait ColumnLocTransform: TableTransform {
        type ColumnTransform;
        type ColumnIterTransform: Iterator<Item = Self::ColumnTransform>;

        fn to_col(self, index: usize) -> Option<Self::ColumnTransform>;
        fn to_cols(self) -> Self::ColumnIterTransform;
    }
}



pub mod sort {
    use super::{ TableMut, TableClone, TableTransform };
    use super::loc:: {
        RowLocView,
        RowLocClone,
        RowLocTransform,
        ColumnLocView,
        ColumnLocClone,
        ColumnLocTransform
    };

    pub trait RowSortMut: TableMut + RowLocView {
        fn sort_rows_mut<K, F>(&mut self, f: F) -> Result<(), &'static str>
            where
                F: Fn(&Self::Row<'_>) -> K,
                K: Ord;
    }

    pub trait RowSortClone: TableClone + RowLocClone + Sized {
        fn sort_rows_clone<K, F>(&self, f: F) -> Result<Self, &'static str>
            where
                F: Fn(&Self::RowClone) -> K,
                K: Ord;
    }

    pub trait RowSortTransform: TableTransform + RowLocTransform + Sized {
        fn sort_rows<K, F>(self, f: F) -> Result<Self, &'static str>
            where
                F: Fn(&Self::RowTransform) -> K,
                K: Ord;
    }



    pub trait ColumnSortMut: TableMut + ColumnLocView {
        fn sort_cols_mut<K, F>(&mut self, f: F) -> Result<(), &'static str>
            where
                F: Fn(&Self::Column<'_>) -> K,
                K: Ord;
    }

    pub trait ColumnSortClone: TableClone + ColumnLocClone + Sized {
        fn sort_cols_clone<K, F>(&self, f: F) -> Result<Self, &'static str>
            where
                F: Fn(&Self::ColumnClone) -> K,
                K: Ord;
    }

    pub trait ColumnSortTransform: TableTransform + ColumnLocTransform + Sized {
        fn sort_cols<K, F>(self, f: F) -> Result<Self, &'static str>
            where
                F: Fn(&Self::ColumnTransform) -> K,
                K: Ord;
    }
}


pub mod map {
    use super::{ TableMut };
    use super::loc:: { RowLocView };

    pub trait MapMut: TableMut + RowLocView {

    }
}
