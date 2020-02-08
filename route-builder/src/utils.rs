use anyhow::Result;

pub fn collect_results<I: IntoIterator<Item = Result<T>>, T>(iter: I) -> Result<Vec<T>> {
    let iter = iter.into_iter();
    let mut result = Vec::with_capacity(iter.size_hint().0);
    for item in iter {
        result.push(item?);
    }
    return Ok(result);
}

pub fn join_results<A, B, OA, OB>(oper_a: A, oper_b: B) -> Result<(OA, OB)>
where
    A: FnOnce() -> Result<OA> + Send,
    B: FnOnce() -> Result<OB> + Send,
    OA: Send,
    OB: Send,
{
    let (a, b) = rayon::join(oper_a, oper_b);
    Ok((a?, b?))
}

pub fn timeit<S, A, RA>(message: S, oper_a: A) -> RA
where
    A: FnOnce() -> RA,
    S: Into<String>,
{
    let before = std::time::Instant::now();
    let a = oper_a();
    let dur = std::time::Instant::now().duration_since(before);
    println!(
        "{}, {:?}, {}",
        message.into().replace(",", " "),
        dur,
        dur.as_secs_f64(),
    );
    a
}
