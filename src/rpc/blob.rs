pub struct BlobUploadResponse {
    pub stage_id: StagedBlob,
}

pub struct StagedBlob(pub String);