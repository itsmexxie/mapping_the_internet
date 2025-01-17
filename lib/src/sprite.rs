use tokio::{fs::File, io};

pub struct Sprite {
    reader: io::BufReader<File>,
}

impl Sprite {
    pub async fn load<P: AsRef<std::path::Path>>(filename: P) -> Result<Sprite, tokio::io::Error> {
        let sprite_file = File::open(filename).await?;

        Ok(Sprite {
            reader: io::BufReader::new(sprite_file),
        })
    }

    pub async fn print(&mut self) -> Result<u64, tokio::io::Error> {
        io::copy(&mut self.reader, &mut io::stdout()).await
    }
}
