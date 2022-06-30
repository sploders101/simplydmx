mod uuid;
use crate::uuid::Uuid;
use fastrand;
use std::{
    fs,
    time::Instant,
    collections::HashMap,
};
use serde_json;
use bincode;

fn main() {

    println!("Creating sample data...");
    let mut start = Instant::now();
    let mut submasters = HashMap::<Uuid, HashMap::<Uuid, HashMap::<Uuid, u16>>>::new();
    for _ in 0..300 {
        let mut submaster = HashMap::<Uuid, HashMap<Uuid, u16>>::new();
        for _ in 0..1311 {
            let uuid = Uuid::new();
            let mut new_hashmap = HashMap::new();
            for _ in 0..50 {
                let new_uuid = Uuid::new();
                new_hashmap.insert(new_uuid, fastrand::u16(..));
            }
            submaster.insert(uuid, new_hashmap);
        }
        submasters.insert(Uuid::new(), submaster);
    }
    let mut elapsed = start.elapsed();
    println!("Sample data creation took {:?}.", elapsed);

    println!("Starting JSON serialization benchmark.");
    start = Instant::now();
    let serialized = serde_json::to_string(&submasters).unwrap();
    elapsed = start.elapsed();
    println!("JSON serialization took {:?}.", elapsed);

    println!("Starting JSON save.");
    start = Instant::now();
    fs::write("testData/test.json", serialized).unwrap();
    elapsed = start.elapsed();
    println!("JSON Save took {:?}.", elapsed);

    drop(submasters);

    println!("Starting JSON read.");
    start = Instant::now();
    let serialized = fs::read_to_string("testData/test.json").unwrap();
    elapsed = start.elapsed();
    println!("JSON read took {:?}", elapsed);

    println!("Starting JSON deserialize.");
    start = Instant::now();
    let submasters: HashMap::<Uuid, HashMap::<Uuid, HashMap::<Uuid, u16>>> = serde_json::from_str(&serialized).unwrap();
    elapsed = start.elapsed();
    println!("JSON deserialize took {:?}", elapsed);

    println!("Starting bincode serialization");
    start = Instant::now();
    let serialized = bincode::serialize(&submasters).unwrap();
    elapsed = start.elapsed();
    println!("Bincode serialization took {:?}.", elapsed);

    println!("Starting bincode save.");
    start = Instant::now();
    fs::write("testData/test.sdmx", serialized).unwrap();
    elapsed = start.elapsed();
    println!("Bincode save took {:?}.", elapsed);

    drop(submasters);

    println!("Starting bincode read.");
    start = Instant::now();
    let serialized = fs::read("testData/test.sdmx").unwrap();
    elapsed = start.elapsed();
    println!("Bincode read took {:?}", elapsed);

    println!("Starting bincode deserialize.");
    start = Instant::now();
    let submasters: HashMap::<Uuid, HashMap::<Uuid, HashMap::<Uuid, u16>>> = bincode::deserialize(&serialized).unwrap();
    elapsed = start.elapsed();
    println!("Bincode deserialize took {:?}", elapsed);

    println!("Starting clone.");
    start = Instant::now();
    let _new_submasters = submasters.clone();
    elapsed = start.elapsed();
    println!("Clone took {:?}.", elapsed);

}
