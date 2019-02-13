use std::cmp::PartialOrd;

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

pub fn avg(numbers: &[f64]) -> f64 {
    numbers.iter().sum::<f64>() as f64 / numbers.len() as f64
}