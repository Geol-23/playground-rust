use std::default::Default;

use super::traits::{ Table, Schema, TableMut };

type Vec2D<T> = Vec<Vec<T>>;

#[derive(Debug)]
pub struct ColumnarTableHeader {
    headers: Vec<String>
}

impl Schema for ColumnarTableHeader {
    type TypeId = ();

    fn nfields(&self) -> usize { self.headers.len() }
    fn field_name(&self, index: usize) -> &str { self.headers[index].as_str() }
    fn field_type(&self, index: usize) -> Self::TypeId { () }
}

#[derive(Debug)]
pub struct ColumnarTable<T: Sized + Clone> {
    data: Vec<Vec<T>>,
    header: ColumnarTableHeader
}
pub type ColTable<T> = ColumnarTable<T>;

impl<T: Default + Clone> ColumnarTable<T> {
    // pub fn new(data: Vec<Vec<T>>) -> Self {
    //     let header = ColumnarTableHeader {
    //         headers: vec![(String::new(), T::default()); data.len()]
    //     };
    //     Self {
    //         data,
    //         header
    //     }
    // }
    //
    pub fn builder() -> ColumnarTableBuilder<T> {
        ColumnarTableBuilder {
            columns: Vec::new(),
            headers: Vec::new()
        }
    }
}

impl<T: Clone + Sized> Table for ColumnarTable<T> {
    type TypeId = ();

    fn nrows(&self) -> usize {
        match self.data.get(0) {
            None => 0,
            Some(d) => d.len()
        }
    }

    fn ncols(&self) -> usize { self.data.len() }
    fn schema(&self) -> &dyn Schema<TypeId = ()> { return &self.header }
}




pub struct ColumnarTableBuilder<T> {
    columns: Vec2D<T>,
    headers: Vec<String>
}

impl<T: Clone> ColumnarTableBuilder<T> {
    pub fn column(mut self: Self, name: impl Into<String>, data: Vec<T>) -> Self {
        self.headers.push(name.into());

        if !self.columns.is_empty() && data.len() != self.columns[0].len() {
            panic!(
                "Ivalid ColumnarTable initialisation. Length of column {} != {}",
                self.headers.last().unwrap(),
                self.columns[0].len()
            )
        }
        self.columns.push(data);
        self
    }

    pub fn build(self) -> ColumnarTable<T> {
        ColumnarTable {
            data: self.columns,
            header: ColumnarTableHeader {
                headers: self.headers
            }
        }
    }
}
