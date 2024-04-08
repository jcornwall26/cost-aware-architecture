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

        // let mut file = File::open(csv_local_path).unwrap();
        // let mut file_data: Vec<u8> = vec![];
        // file.read_to_end(&mut file_data).unwrap();
        //ByteStream::new()

        let stream = ByteStream::read_from()
            .path(csv_local_path)
            .build()
            .await
            .unwrap();

        match self.aws_client.put_object()
            .key("temp_key")
            .bucket("bucket")
            .body(stream)
            .send().await
            {
                Ok(_) => (),
                Err(e) => println!("Put object failed: {:?}", e)
            }
    }
}