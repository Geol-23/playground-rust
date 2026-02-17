use std::any::{ Any, TypeId };
use std::fmt;
use std::iter::{ ExactSizeIterator, FusedIterator };
use std::ops::{ Index, Range };

use super::traits::{ Table, Schema };
use super::traits::loc::{ RowLocView };

// *************************************************************************
//                                  SCHEMA
// *************************************************************************
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


// *************************************************************************
//                                  COLUMN
// *************************************************************************
pub trait Column: fmt::Debug {
    fn len(&self) -> usize;
    fn type_id(&self) -> TypeId;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn fmt_cell(&self, row: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn cell_to_string(&self, row: usize) -> String;
}
type DynColumn = Box<dyn Column>;

#[derive(Debug)]
pub struct IMCTColumn<T: fmt::Debug> {
    data: Vec<T>,
}
impl<T: 'static + fmt::Debug + fmt::Display> Column for IMCTColumn<T> {
    fn len(&self) -> usize { self.data.len() }
    fn type_id(&self) -> TypeId { TypeId::of::<T>() }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn fmt_cell(&self, row: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data[row])
    }
    fn cell_to_string(&self, row: usize) -> String { format!("{}", self.data[row]) }
}


// *************************************************************************
//                            InMemoryColumnTable
// *************************************************************************
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
impl fmt::Display for InMemoryColumnTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let terminal_width: usize = crossterm::terminal::size()
                    .map(|ts| ts.0)
                    .unwrap_or(80).into();
        let renderer = IMCTTerminalRenderer { terminal_width };
        match renderer.layout(self) {
            Err(layout_err) => {
                match layout_err {
                    IMCTTableLayoutError::EmptyTable => writeln!(f, "<empty table>"),
                    IMCTTableLayoutError::OversizedColumn => writeln!(f, "<{}x{} table>", self.ncols(), self.nrows()),
                }
            },
            Ok(layout) => {
                renderer.render(self, &layout, f)
            }
        }
    }
}

// *************************************************************************
//                                IMCTLayout
// *************************************************************************
pub enum IMCTTableLayoutError {
    EmptyTable,
    OversizedColumn
}
pub struct IMCTTableLayout {
    lrange: Range<usize>,
    rrange: Range<usize>,
    raw_column_widths: Vec<usize>,
    ellipsis: bool,
}
impl IMCTTableLayout {
    pub fn new(
        lrange: Range<usize>,
        rrange: Range<usize>,
        raw_column_widths: Vec<usize>,
        ellipsis: bool
    ) -> Self {
        Self {
            lrange,
            rrange,
            raw_column_widths,
            ellipsis
        }
    }
    pub fn visible_columns(&self) -> impl Iterator<Item = usize> + '_ {
        let left = self.lrange.clone();
        let right = self.rrange.clone();
        left.chain(right)
    }
}
pub trait IMCTTableRenderer {
    fn layout(&self, table: &InMemoryColumnTable) -> Result<IMCTTableLayout, IMCTTableLayoutError>;
    fn render(&self, table: &InMemoryColumnTable, layout: &IMCTTableLayout, f: &mut fmt::Formatter) -> fmt::Result;
}
struct IMCTTerminalRenderer {
    terminal_width: usize,
}
impl IMCTTerminalRenderer {
    pub fn render_with_ellipsis(&self,
        table: &InMemoryColumnTable,
        layout: &IMCTTableLayout,
        f: &mut fmt::Formatter
    ) -> fmt::Result {

        fn render_target_row<T, U>(
            f: &mut fmt::Formatter,
            layout: &IMCTTableLayout,
            target: T
        ) -> fmt::Result
        where
            T: Index<usize, Output = U>,
            U: fmt::Display
        {
            for i in layout.lrange.clone() {
                if i > 0 {
                    write!(f, "|")?;
                }
                write!(f, " {:width$} ", target[i], width=layout.raw_column_widths[i])?;
            }
            write!(f, "| ... ")?;
            for i in layout.rrange.clone() {
                if i > 0 {
                    write!(f, "|")?;
                }
                write!(f, " {:width$} ", target[i], width=layout.raw_column_widths[i])?;
            }
            writeln!(f)
        }

        let header_names: Vec<&String> = table
            .header.fields.iter()
            .map(|f| { &f.name })
            .collect();
        let pads: Vec<String> = layout
            .raw_column_widths.iter()
            .map(|l| "-".repeat(*l))
            .collect();
        render_target_row(f, layout, header_names)?;
        render_target_row(f, layout, pads)?;
        for j in 0..table.nrows() {
            let mut row: Vec<String> = Vec::new();
            for i in 0..table.ncols() {
                row.push(table.columns[i].cell_to_string(j));
            }
            render_target_row(f, layout, row)?;
        }
        Ok(())
    }

    pub fn render_no_ellipsis(&self,
        table: &InMemoryColumnTable,
        layout: &IMCTTableLayout,
        f: &mut fmt::Formatter
    ) -> fmt::Result {
        let col_widths = &layout.raw_column_widths;
        let fields = &table.header.fields;

        // Writing the header
        for i in layout.visible_columns() {
            if i > 0 {
                write!(f, "|")?;
            }
            write!(f, " {:width$} ", fields[i].name, width=col_widths[i])?;
        }
        writeln!(f)?;

        // Writing the separator
        for i in layout.visible_columns() {
            if i > 0 {
                write!(f, "|")?;
            }
            write!(f, " {:-<width$} ", "", width=col_widths[i])?;
        }
        writeln!(f)?;

        // Writing the data
        for j in 0..table.nrows() {
            for i in layout.visible_columns() {
                if i > 0 {
                    write!(f, "|")?;
                }
                let s = table.columns[i].cell_to_string(j);
                write!(f, " {:width$} ", s, width=col_widths[i])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
struct Splitter {
    p: usize,
    s: usize,
    polarity: bool,
    ncols: usize,
}
impl Splitter {
    fn p(&self) -> usize { self.p }
    fn s(&self) -> usize { self.ncols - self.s }
    fn new(ncols: usize) -> Self {
        let p = ncols / 2;
        let s = ncols - p;
        let polarity = s > p;
        Self {
            p,
            s,
            polarity,
            ncols,
        }
    }
}
impl Iterator for Splitter {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.p + self.s == 0 { return None }
        if self.polarity {
            self.s -= 1;
        } else {
            self.p -= 1;
        }
        self.polarity = !self.polarity;
        Some((self.p(), self.s()))
    }
}
impl IMCTTableRenderer for IMCTTerminalRenderer {
    fn layout(&self, table: &InMemoryColumnTable) -> Result<IMCTTableLayout, IMCTTableLayoutError> {
        const ELLIPSIS_WIDTH: usize = 3;

        let ncols = table.ncols();
        let nrows = table.nrows();

        if ncols == 0 {
            return Err(IMCTTableLayoutError::EmptyTable)
        }

        let mut col_widths = vec![0_usize; ncols];
        for i in 0..ncols {
            col_widths[i] = table.header.field_name(i).len();
            for j in 0..nrows {
                let len = table.columns[i].cell_to_string(j).len();
                col_widths[i] = col_widths[i].max(len);
            }
        }

        let mut total = 0;
        let mut p_widths: Vec<usize> = col_widths.iter()
            .map(|i| { total += i; total })
            .collect();
        p_widths.insert(0, 0);
        let total_raw = p_widths[ncols];
        let rendered_column_width = |p_widths: &[usize], p: usize, s: usize, ellipsis: bool| {
            let el: usize = ellipsis as usize;
            p_widths[p] + total_raw - p_widths[s] + ELLIPSIS_WIDTH * el + (p + s + el) * 3 - 1
        };
        let splitter = Splitter::new(ncols);
        // println!("p: {}, s: {}", splitter.p(), splitter.s());
        // println!("pwidths: {:?}", p_widths);
        // println!("Rendered width: {}", rendered_column_width(&p_widths, splitter.p(), splitter.s(), false));
        // println!("Rendered width (ellipse): {}", rendered_column_width(&p_widths, splitter.p(), splitter.s(), true));
        // println!("Terminal width: {}", self.terminal_width);
        // println!("prefix indicies: {:?}, suffix indicies: {:?}", 0..splitter.p()+1, splitter.s()..ncols);


        if rendered_column_width(&p_widths, splitter.p(), splitter.s(), false) <= self.terminal_width {
            return Ok(IMCTTableLayout::new(0..splitter.p()+1, splitter.s()+1..ncols, col_widths, false))
        }

        // println!("p: {}, s: {}", splitter.p(), splitter.s());
        for (p, s) in splitter {
            // println!("p: {}, s: {}", p, s);
            // println!("Rendered width: {}", rendered_column_width(&p_widths, p, s, true));
            if (p, s) == (0, ncols) {
                return Err(IMCTTableLayoutError::OversizedColumn)
            }
            if rendered_column_width(&p_widths, p, s, true) <= self.terminal_width {
                return Ok(IMCTTableLayout::new(0..p+1, s+1..ncols, col_widths, true))
            }
        }
        return Err(IMCTTableLayoutError::OversizedColumn)
    }
    fn render(&self,
        table: &InMemoryColumnTable,
        layout: &IMCTTableLayout,
        f: &mut fmt::Formatter
    ) -> fmt::Result {
        if layout.ellipsis {
            self.render_with_ellipsis(table, layout, f)
        } else {
            self.render_no_ellipsis(table, layout, f)
        }
    }
}



// *************************************************************************
//                                RowLocView
// *************************************************************************
pub struct IMCTRowView<'a> {
    source: &'a InMemoryColumnTable,
    index: usize,
}
impl<'a> IMCTRowView<'a> {
    pub fn get<T: fmt::Debug + 'static>(&self, index: usize) -> Option<&'a T> {
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


// *************************************************************************
//                       InMemoryColumnarTableBuilder
// *************************************************************************
pub struct IMCTBuilder {
    columns: Vec<DynColumn>,
    fields: Vec<Field>,
}
impl IMCTBuilder {
    pub fn column<T: 'static + fmt::Debug + fmt::Display>(mut self, name: impl Into<String>, data: Vec<T>) -> Self {
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
