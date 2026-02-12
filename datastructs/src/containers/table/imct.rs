use std::any::{ Any, TypeId };
use std::fmt::Debug;
use std::iter::{ ExactSizeIterator, FusedIterator };

use super::traits::{ Table, Schema };
use super::traits::loc::{ RowLocView };


#[derive(Debug)]
pub struct Field {
    name: String,
    type_id: TypeId,
}

#[derive(Debug)]
pub struct IMCTHeader {
    fields: Vec<Field>,
}

impl Schema for IMCTHeader {
    fn nfields(&self) -> usize { self.fields.len() }
    fn field_name(&self, index: usize) -> &str { &self.fields[index].name }
    fn field_type(&self, index: usize) -> TypeId { self.fields[index].type_id }
}


trait Column: Debug {
    fn len(&self) -> usize;
    fn type_id(&self) -> TypeId;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
type DynColumn = Box<dyn Column>;

#[derive(Debug)]
pub struct IMCTColumn<T: Debug> {
    data: Vec<T>,
}

impl<T: 'static + Debug> Column for IMCTColumn<T> {
    fn len(&self) -> usize { self.data.len() }
    fn type_id(&self) -> TypeId { TypeId::of::<T>() }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}



#[derive(Debug)]
pub struct InMemoryColumnTable {
    columns: Vec<DynColumn>,
    header: IMCTHeader,
}
pub type ColTable = InMemoryColumnTable;

impl InMemoryColumnTable {
    pub fn builder() -> IMCTBuilder {
        IMCTBuilder {
            columns: Vec::new(),
            fields: Vec::new()
        }
    }
}


impl Table for InMemoryColumnTable {
    fn nrows(&self) -> usize { self.columns.first().map_or(0, |c| c.len()) }
    fn ncols(&self) -> usize { self.columns.len() }
    fn schema(&self) -> &dyn Schema { &self.header }
}


pub struct IMCTRowView<'a> {
    source: &'a InMemoryColumnTable,
    index: usize,
}
impl<'a> IMCTRowView<'a> {
    pub fn get<T: Debug + 'static>(&self, index: usize) -> Option<&'a T> {
        let col = self.source
            .columns
            .get(index)?
            .as_any()
            .downcast_ref::<IMCTColumn<T>>()?;

        col.data.get(self.index)
    }
}

pub struct IMCTRowViewIter<'a> {
    source: &'a InMemoryColumnTable,
    curr_i: usize,
}
impl<'a> IMCTRowViewIter<'a> {
    fn new(source: &'a InMemoryColumnTable) -> Self {
        Self {
            source,
            curr_i: 0
        }
    }
}
impl<'a> Iterator for IMCTRowViewIter<'a> {
    type Item = IMCTRowView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_i < self.source.nrows() {
            let row = IMCTRowView {
                source: self.source,
                index: self.curr_i,
            };
            self.curr_i += 1;
            Some(row)
        } else {
            None
        }
    }
}
impl<'a> IntoIterator for &'a InMemoryColumnTable {
    type Item = IMCTRowView<'a>;
    type IntoIter = IMCTRowViewIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IMCTRowViewIter::new(self)
    }
}
impl<'a> FusedIterator for IMCTRowViewIter<'a> {}
impl<'a> ExactSizeIterator for IMCTRowViewIter<'a> {
    fn len(&self) -> usize { self.source.nrows() - self.curr_i }
    // Removed due to unstable implementation (nightly)
    // fn is_empty(&self) -> bool { self.curr_i >= self.len() }
}


impl RowLocView for InMemoryColumnTable {
    type Row<'a> = IMCTRowView<'a>;
    type RowIter<'a> = IMCTRowViewIter<'a>;

    fn row(&self, index: usize) -> Option<Self::Row<'_>> {
        (index < self.nrows()).then(|| Self::Row {
            source: self,
            index
        })
    }
    fn rows(&self) -> Self::RowIter<'_> {
        self.into_iter()
    }
}


pub struct IMCTBuilder {
    columns: Vec<DynColumn>,
    fields: Vec<Field>,
}

impl IMCTBuilder {
    pub fn column<T: 'static + Debug>(mut self, name: impl Into<String>, data: Vec<T>) -> Self {
        let type_id = TypeId::of::<T>();
        let name: String = name.into();

        if let Some(first) = self.columns.first() {
            let expected = first.len();
            if data.len() != expected {
                panic!(
                    "Invalid IMCT initialization: column '{}' length {} != {}",
                    name,
                    data.len(),
                    expected
                );
            }
        }


        self.fields.push( Field {
            name,
            type_id
        });
        self.columns.push(Box::new(IMCTColumn { data }));
        self
    }

    pub fn build(self) -> InMemoryColumnTable {
        InMemoryColumnTable {
            columns: self.columns,
            header: IMCTHeader {
                fields: self.fields
            }
        }
    }
}
