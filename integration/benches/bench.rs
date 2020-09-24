#![feature(test)]

use integration::*;

use std::thread;
use std::time::Duration;

extern crate test;

fn do_bench_commit(n: u8, b: &mut test::Bencher) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let env = init_cluster(n);
    let id = env.node_id(0);
    b.iter(|| {
        let endpoint = lol_core::connection::Endpoint::new(id.clone());
        let msg = kvs::Req::Set {
            key: "k".to_owned(),
            value: "v".to_owned(),
        };
        let msg = kvs::Req::serialize(&msg);
        let r = rt.block_on(async move {
            let mut conn = endpoint.connect().await.unwrap();
            conn.request_commit(lol_core::protoimpl::CommitReq {
                core: false,
                message: msg,
            })
            .await
        });
        assert!(r.is_ok());
    })
}
#[bench]
fn bench_commit_1(b: &mut test::Bencher) {
    do_bench_commit(1, b)
}
#[bench]
fn bench_commit_4(b: &mut test::Bencher) {
    do_bench_commit(4, b)
}
#[bench]
fn bench_commit_16(b: &mut test::Bencher) {
    do_bench_commit(16, b)
}
#[bench]
fn bench_commit_64(b: &mut test::Bencher) {
    do_bench_commit(64, b)
}

fn do_bench_apply(n: u8, b: &mut test::Bencher) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let env = init_cluster(n);
    Client::to(0, env.clone()).set("k", "v");
    thread::sleep(Duration::from_secs(1));

    let id = env.node_id(0);
    b.iter(|| {
        let endpoint = lol_core::connection::Endpoint::new(id.clone());
        let msg = kvs::Req::Get {
            key: "k".to_owned(),
        };
        let msg = kvs::Req::serialize(&msg);
        let r = rt.block_on(async move {
            let mut conn = endpoint.connect().await.unwrap();
            conn.request_apply(lol_core::protoimpl::ApplyReq {
                core: false,
                mutation: true,
                message: msg,
            })
            .await
        });
        assert!(r.is_ok());
    })
}
#[bench]
fn bench_apply_1(b: &mut test::Bencher) {
    do_bench_apply(1, b)
}
#[bench]
fn bench_apply_4(b: &mut test::Bencher) {
    do_bench_apply(4, b)
}
#[bench]
fn bench_apply_16(b: &mut test::Bencher) {
    do_bench_apply(16, b)
}
#[bench]
fn bench_apply_64(b: &mut test::Bencher) {
    do_bench_apply(64, b)
}

fn do_bench_query(n: u8, b: &mut test::Bencher) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let env = init_cluster(n);
    Client::to(0, env.clone()).set("k", "v");
    thread::sleep(Duration::from_secs(1));

    let id = env.node_id(0);
    b.iter(|| {
        let endpoint = lol_core::connection::Endpoint::new(id.clone());
        let msg = kvs::Req::Get {
            key: "k".to_owned(),
        };
        let msg = kvs::Req::serialize(&msg);
        let r = rt.block_on(async move {
            let mut conn = endpoint.connect().await.unwrap();
            conn.request_apply(lol_core::protoimpl::ApplyReq {
                core: false,
                mutation: false,
                message: msg,
            })
            .await
        });
        assert!(r.is_ok());
    })
}
#[bench]
fn bench_query_1(b: &mut test::Bencher) {
    do_bench_query(1, b);
}
#[bench]
fn bench_query_4(b: &mut test::Bencher) {
    do_bench_query(4, b);
}
#[bench]
fn bench_query_16(b: &mut test::Bencher) {
    do_bench_query(16, b);
}
#[bench]
fn bench_query_64(b: &mut test::Bencher) {
    do_bench_query(64, b);
}

fn do_bench_commit_huge(n: u8, b: &mut test::Bencher) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let env = init_cluster(n);
    let id = env.node_id(0);
    // 100KB
    let mut v = String::new();
    for _ in 0..100_000 {
        v.push('a');
    }
    b.iter(|| {
        let endpoint = lol_core::connection::Endpoint::new(id.clone());
        let msg = kvs::Req::Set {
            key: "k".to_owned(),
            value: v.clone(),
        };
        let msg = kvs::Req::serialize(&msg);
        let r = rt.block_on(async move {
            let mut conn = endpoint.connect().await.unwrap();
            conn.request_commit(lol_core::protoimpl::CommitReq {
                core: false,
                message: msg,
            })
            .await
        });
        assert!(r.is_ok());
    })
}
#[bench]
fn test_commit_huge_1(b: &mut test::Bencher) {
    do_bench_commit_huge(1, b)
}
#[bench]
fn test_commit_huge_4(b: &mut test::Bencher) {
    do_bench_commit_huge(4, b)
}
#[bench]
fn test_commit_huge_16(b: &mut test::Bencher) {
    do_bench_commit_huge(16, b)
}
#[bench]
fn test_commit_huge_64(b: &mut test::Bencher) {
    do_bench_commit_huge(64, b)
}