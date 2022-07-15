use crate::Exception;
use tokio::io::AsyncReadExt;

pub struct Font {}

impl Font {
    pub async fn load(path: String) -> Result<Self, Exception> {
        println!("Loading font {}", path);

        let file = tokio::fs::File::open(path).await;
        if let Err(_e) = file {
            return Err(Exception::IoError);
        }
        let mut file = file.unwrap();
        if file.metadata().await?.len() as u64 != 1140 {
            return Err(Exception::ContentError);
        }

        for _character in 0..95 {
            for _row in 0..12 {
                let _data = file.read_u8();
            }
        }
        Ok(Self {})
    }
}
