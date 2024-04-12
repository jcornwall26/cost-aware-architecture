use aws_config::SdkConfig;
use aws_smithy_types::byte_stream::ByteStream;

pub struct S3Client {
    aws_client: aws_sdk_s3::Client
}

impl S3Client {
    pub fn new(shared_config: SdkConfig) -> S3Client {
        S3Client {
            aws_client: aws_sdk_s3::Client::new(&shared_config)
        }
    }

    pub async fn upload_csv(&self, csv_local_path: &str) {

        let stream = ByteStream::read_from()
            .path(csv_local_path)
            .build()
            .await
            .unwrap();

        let obj_key = build_object_key_from_csv_path(csv_local_path);

        match self.aws_client.put_object()
            .key(obj_key)
            .bucket("athena-jsc-poc")
            .body(stream)
            .send().await
            {
                Ok(_) => (),
                Err(e) => println!("Put object failed: {:?}", e)
            }
    }
}

fn build_object_key_from_csv_path(csv_local_path: &str) -> String {
    // cut off suffix & leading ./
    let len = csv_local_path.find("-output.csv").unwrap();
    let obj_suffix = csv_local_path.get(2..len).unwrap();
    format!("lambda-costs/{}.csv", obj_suffix)
}

#[cfg(test)]
mod tests {

    use super::build_object_key_from_csv_path;

    #[test]
    fn build_object_key_from_csv_path_tests() {
        let path = "./this-is-an-object-key-output.csv";
        let key = build_object_key_from_csv_path(path);
        assert!(key == "lambda-costs/this-is-an-object-key.csv")
    }
}