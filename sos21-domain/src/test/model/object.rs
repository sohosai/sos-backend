use crate::model::object::{data::ObjectDataSummaryReceiver, Object, ObjectData, ObjectId};
use bytes::Bytes;
use futures::stream;
use rand::{distributions::Standard, Rng};
use uuid::Uuid;

pub fn new_object_id() -> ObjectId {
    ObjectId::from_uuid(Uuid::new_v4())
}

pub fn new_object_data_with_size_and_summary(
    size: u64,
) -> (ObjectData, [u8; 32], ObjectDataSummaryReceiver) {
    let mut rng = rand::thread_rng();
    let mut hasher = blake3::Hasher::new();

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
            .inspect(|bytes: &Bytes| {
                hasher.update(bytes);
            })
            .collect();

    let digest = hasher.finalize().into();
    let (object_data, receiver) =
        ObjectData::from_stream_with_summary(stream::iter(data.into_iter().map(Ok)));
    (object_data, digest, receiver)
}

pub fn new_object_data_with_size(size: u64) -> (ObjectData, [u8; 32]) {
    let (object_data, digest, _) = new_object_data_with_size_and_summary(size);
    (object_data, digest)
}

pub fn new_object_data() -> (ObjectData, [u8; 32], u64) {
    let size = rand::thread_rng().gen_range(1..2048);
    let (object_data, digest) = new_object_data_with_size(size);
    (object_data, digest, size)
}

pub fn new_object_data_with_summary() -> (ObjectData, [u8; 32], u64, ObjectDataSummaryReceiver) {
    let size = rand::thread_rng().gen_range(1..2048);
    let (object_data, digest, receiver) = new_object_data_with_size_and_summary(size);
    (object_data, digest, size, receiver)
}

pub fn new_object_with_size(size: u64) -> (Object, [u8; 32]) {
    let (data, digest) = new_object_data_with_size(size);
    (
        Object {
            id: new_object_id(),
            data,
        },
        digest,
    )
}

pub fn new_object() -> (Object, [u8; 32], u64) {
    let size = rand::thread_rng().gen_range(1..2048);
    let (object, digest) = new_object_with_size(size);
    (object, digest, size)
}
