use tokio::net::TcpStream;
use tokio::io::{ AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader };
use tokio::sync::mpsc;
use std::error::Error;
use std::io::{ self, Write };
use std::thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server_addr = "127.0.0.1:8080";
    let server_stream = TcpStream::connect(server_addr).await?;
    println!("Sunucuya bağlanıldı: {}", server_addr);

    let (reader, mut writer) = server_stream.into_split();
    let mut reader = BufReader::new(reader);

    let (tx, mut rx) = mpsc::channel::<String>(10);

    let mut initial_buffer = [0; 1024];

    match reader.read(&mut initial_buffer).await {
        Ok(n) if n > 0 => println!("{}", String::from_utf8_lossy(&initial_buffer)),
        _ => {}
    }

    print!("> ");
    io::stdout().flush()?;

    let read_task = tokio::spawn(async move {
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("Sunucu bağlantısı kapandı.");
                    io::stdout().flush().unwrap();
                    return;
                }
                Ok(_) => {
                    print!("\r{}", line.trim_end());
                    println!("> ");
                    io::stdout().flush().unwrap();
                    line.clear();
                }
                Err(e) => {
                    eprintln!("Mesaj okunamadı: {}", e);
                    io::stdout().flush().unwrap();
                    break;
                }
            }
        }
    });

    let write_task = tokio::spawn(async move {
        loop {
            let mut user_input = String::new();
            print!(">");
            io::stdout().flush().unwrap();

            if io::stdin().read_line(&mut user_input).is_err() {
                eprintln!("Girdi okunamadı!");
                break;
            }

            if writer.write_all(user_input.as_bytes()).await.is_err() {
                eprintln!("Girdi sunucuya gönderilemedi.");
                break;
            }

            if user_input.trim().eq_ignore_ascii_case("/quit") {
                print!("Çıkılıyor");
                break;
            }
        }
    });

    let _ = tokio::try_join!(read_task, write_task);

    Ok(())
}
