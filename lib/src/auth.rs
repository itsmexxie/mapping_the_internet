use tokio::{fs::File, io::AsyncReadExt};

pub struct JWTKeys {
    pub private: Option<Vec<u8>>,
    pub public: Vec<u8>,
}

pub enum JWTKeyError {
    FileNotFound,
    FailedRead,
}

impl JWTKeys {
    async fn load_file(filename: &str) -> Result<Vec<u8>, JWTKeyError> {
        let mut key_file = match File::open(filename).await {
            Ok(key_file) => key_file,
            Err(_) => return Err(JWTKeyError::FileNotFound),
        };

        let mut key = Vec::new();

        match key_file.read_to_end(&mut key).await {
            Ok(_) => Ok(key),
            Err(_) => Err(JWTKeyError::FailedRead),
        }
    }

    pub async fn load(private_filename: &str, public_filename: &str) -> Self {
        let private_key = match JWTKeys::load_file(private_filename).await {
            Ok(key) => key,
            Err(error) => match error {
                JWTKeyError::FileNotFound => panic!("File {} not found!", private_filename),
                JWTKeyError::FailedRead => panic!("Failed to read file {}!", private_filename),
            },
        };

        let public_key = match JWTKeys::load_file(public_filename).await {
            Ok(key) => key,
            Err(error) => match error {
                JWTKeyError::FileNotFound => panic!("File {} not found!", public_filename),
                JWTKeyError::FailedRead => panic!("Failed to read file {}!", public_filename),
            },
        };

        JWTKeys {
            private: Some(private_key),
            public: public_key,
        }
    }

    pub async fn load_public(public_filename: &str) -> Self {
        let public_key = match JWTKeys::load_file(public_filename).await {
            Ok(key) => key,
            Err(error) => match error {
                JWTKeyError::FileNotFound => panic!("File {} not found!", public_filename),
                JWTKeyError::FailedRead => panic!("Failed to read file {}!", public_filename),
            },
        };

        JWTKeys {
            private: None,
            public: public_key,
        }
    }
}
