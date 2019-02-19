use std::cmp::PartialOrd;
use std::iter::Sum;

pub fn min<T : PartialOrd>(numbers: &[T]) -> &T {
    let mut i = numbers.iter();
    let mut m = i.next().unwrap();

    while let Some(n) = i.next() {
        if n < m { m = n; }
    }

    m
}

pub fn max<T : PartialOrd>(numbers: &[T]) -> &T {
    let mut i = numbers.iter();
    let mut m = i.next().unwrap();

    while let Some(n) = i.next() {
        if n > m { m = n; }
    }

    m
}

pub fn avg<T>(numbers: &[T]) -> f64 where T : Copy + Sum, f64 : From<T> {
    let sum: T = numbers.iter().map(|n| n.to_owned()).sum();
    f64::from(sum) / numbers.len() as f64
}