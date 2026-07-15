// Standard imports
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::slice;
use std::str::FromStr;

#[derive(Debug)]
struct ArgDecodeError;
impl fmt::Display for ArgDecodeError {
    fn fmt(self: &Self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error decoding argument")
    }
}
impl std::error::Error for ArgDecodeError {}

type ArgumentGroup = Vec<String>;
#[derive(Debug)]
pub struct ArgumentTable {
    pub name: String,
    pub cmd: Option<Box<ArgumentTable>>,
    pub args: HashMap<String, Vec<ArgumentGroup>>,
}

struct ArgIter<'a, T> {
    inner: slice::Iter<'a, String>,
    _marker: PhantomData<T>,
}
impl<'a, T> Iterator for ArgIter<'a, T>
where
    T: FromStr
{
    type Item = Result<T, ArgDecodeError>;

    fn next(self: &mut Self) -> Option<Self::Item> {
        self.inner.next().map(|val| {
            T::from_str(val).map_err(|_| ArgDecodeError {})
        })
    }
}

struct ArgGroupIter<'a, T> {
    inner: slice::Iter<'a, Vec<String>>,
    _marker: PhantomData<T>,
}
impl<'a, T> Iterator for ArgGroupIter<'a, T>
where
    T: FromStr
{
    type Item = ArgIter<'a, T>;

    fn next(self: &mut Self) -> Option<Self::Item> {
        self.inner.next()
            .and_then(|vec| {
                Some(
                    ArgIter {
                        inner: vec.iter(),
                        _marker: PhantomData::<T>,
                    }
                )
            })
    }
}


impl ArgumentTable {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cmd: None,
            args: HashMap::new(),
        }
    }
    pub fn subcommand(self: &Self) -> Option<(&str, &ArgumentTable)> {
        self.cmd.as_ref().map(|cmd| (self.name.as_str(), cmd.as_ref()))
    }
    pub fn get_first<'a, 'b, T, U>(self: &'a Self, dest: U) -> Result<Option<T>, ArgDecodeError>
    where
        T: FromStr,
        U: AsRef<str>,
    {
        self.args
            .get(dest.as_ref())
            .and_then(|groups: &Vec<ArgumentGroup>| {
                groups.first()
                    .and_then(|group: &ArgumentGroup| {
                        group.first()
                            .map(|v| {
                                T::from_str(v).map_err(|_| ArgDecodeError)
                            })
                    })
            })
            .transpose()
    }
    pub fn get_first_group<'a, 'b, T, U>(self: &'a Self, dest: U) -> Option<ArgIter<'a, T>>
    where
        T: FromStr,
        U: AsRef<str>,
    {
        self.args
            .get(dest.as_ref())
            .and_then(|groups: &Vec<ArgumentGroup>| {
                groups.first()
                    .and_then(|group: &ArgumentGroup| {
                        Some(
                            ArgIter {
                                inner: group.iter(),
                                _marker: PhantomData::<T>,
                            }
                        )
                    })
            })
    }
    pub fn get_groups<'a, 'b, T, U>(self: &'a Self, dest: U) -> Option<ArgGroupIter<'a, T>>
    where
        T: FromStr,
        U: AsRef<str>,
    {
        self.args
            .get(dest.as_ref())
            .and_then(|groups: &Vec<ArgumentGroup>| {
                Some(
                    ArgGroupIter {
                        inner: groups.iter(),
                        _marker: PhantomData::<T>,
                    }
                )
            })
    }
}
