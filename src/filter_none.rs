pub fn filter_none<'a, I, T>(it: I) -> impl Iterator<Item = &'a T>
where
    I: Iterator<Item = &'a Option<T>>,
    T: 'a,
{
    it.filter_map(|opt| {
        match opt {
            Some(val) => Some(val),
            None => None,
        }
    })
}

pub fn filter_none_mut<'a, I, T>(it: I) -> impl Iterator<Item = &'a mut T>
    where
        I: Iterator<Item = &'a mut Option<T>>,
        T: 'a,
{
    it.filter_map(|opt| {
        match opt {
            Some(val) => Some(val),
            None => None,
        }
    })
}
