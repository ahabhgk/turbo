#![feature(min_specialization)]

use anyhow::{anyhow, Result};
use turbo_tasks::{primitives::StringVc, Value, ValueToString, ValueToStringVc};
use turbo_tasks_testing::{register, run};

register!();

#[tokio::test]
async fn demo() -> Result<()> {
    run! {
        let a1 = AVc::new(1);
        let b1 = BVc::new(2);
        let a2 = AVc::new(2);
        let b2 = BVc::new(1);
        let c1 = sub(a1, b1);
        let c2 = sub(a2, b2);
        let c3 = sub(a1, b1);
        let c4 = sub(a2, b2);
        dbg!(c1);
        dbg!(c3);
        dbg!(c1.resolve().await?);
        dbg!(c3.resolve().await?);
        dbg!(c2 == c4);
    }
    Ok(())
}

#[turbo_tasks::value]
#[derive(Debug)]
struct A {
    inner: usize,
}

impl AVc {
    pub fn new(inner: usize) -> Self {
        Self::cell(A { inner })
    }
}

#[turbo_tasks::value]
#[derive(Debug)]
struct B(usize);

impl BVc {
    pub fn new(inner: usize) -> Self {
        Self::cell(B(inner))
    }
}

#[turbo_tasks::value]
#[derive(Debug)]
enum C {
    Good(usize),
    Bad,
}

#[turbo_tasks::function]
async fn sub(a: AVc, b: BVc) -> Result<CVc> {
    println!("sub: {:?} - {:?}", a, b);
    let a = a.await?.inner;
    let b = b.await?.0;
    if a < b {
        Ok(CVc::cell(C::Bad))
    } else {
        Ok(CVc::cell(C::Good(a - b)))
    }
}
