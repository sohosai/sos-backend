use crate::model::object::{Object, ObjectData, ObjectId};
use bytes::Bytes;
use futures::stream;
use rand::{distributions::Standard, Rng};
use uuid::Uuid;

pub fn new_object_id() -> ObjectId {
    ObjectId::from_uuid(Uuid::new_v4())
}

pub fn new_object_data_with_size(size: u64) -> ObjectData {
    let mut rng = rand::thread_rng();

    let chunk_len = rng.gen_range(1..8);
    let chunk_size = (size as f64 / chunk_len as f64).floor() as usize;
    let last_chunk_size = size as usize - (chunk_size * chunk_len);

    let last_chunk = (&mut rng)
        .sample_iter(Standard)
        .take(last_chunk_size)
        .collect();
    let data: Vec<Bytes> =
        std::iter::repeat_with(|| (&mut rng).sample_iter(Standard).take(chunk_size).collect())
            .take(chunk_len)
            .chain(std::iter::once(last_chunk))
            .collect();

    ObjectData::from_stream_with_size(stream::iter(data.into_iter().map(Ok)), size)
}

pub fn new_object_data() -> (ObjectData, u64) {
    let size = rand::thread_rng().gen_range(1..2048);
    (new_object_data_with_size(size), size)
}

pub fn new_object_with_size(size: u64) -> Object {
    Object {
        id: new_object_id(),
        data: new_object_data_with_size(size),
    }
}

pub fn new_object() -> (Object, u64) {
    let size = rand::thread_rng().gen_range(1..2048);
    (new_object_with_size(size), size)
}
