use super::*;
use aws_sdk_s3::types::*;
use rand::{thread_rng, Rng};

/// Generate random name for a bucket.
fn random_bucket() -> String {
    let mut rng = thread_rng();
    (0..10).map(|_| rng.gen_range('a'..='z')).collect()
}

/// Delete bucket.
async fn delete_bucket(client: Client, bucket: String) {
    let objects = client
        .list_objects_v2()
        .bucket(&bucket)
        .send()
        .await
        .unwrap();

    let mut delete_objects: Vec<ObjectIdentifier> = vec![];
    for obj in objects.contents() {
        let obj_id = ObjectIdentifier::builder()
            .set_key(Some(obj.key().unwrap().to_string()))
            .build()
            .unwrap();
        delete_objects.push(obj_id);
    }

    if !delete_objects.is_empty() {
        client
            .delete_objects()
            .bucket(&bucket)
            .delete(
                Delete::builder()
                    .set_objects(Some(delete_objects))
                    .build()
                    .unwrap(),
            )
            .send()
            .await
            .unwrap();
    }

    client.delete_bucket().bucket(bucket).send().await.unwrap();
}

impl S3 {
    /// Create test client for S3.
    pub async fn new_temp() -> Temporary<S3> {
        use std::env::var;
        let options = S3Options {
            storage_s3_endpoint: Some(var("MINIO_ENDPOINT").unwrap().parse().unwrap()),
            storage_s3_access_key_id: Some(var("MINIO_USER").unwrap()),
            storage_s3_secret_access_key: Some(var("MINIO_PASS").unwrap()),
            storage_s3_region: Some("us-east-1".into()),
            storage_s3_path_style: true,
            storage_s3_bucket: Some(random_bucket()),
        };
        let s3 = options.build().await;
        println!("Using client {s3:?}");
        s3.client()
            .create_bucket()
            .bucket(s3.bucket())
            .send()
            .await
            .unwrap();
        let cleanup = Box::pin(delete_bucket(s3.client().clone(), s3.bucket().into()));
        Temporary::new(s3, cleanup)
    }
}
