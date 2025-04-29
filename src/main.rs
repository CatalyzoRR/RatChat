use crossterm::{
    event::{ self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use ratatui::{ backend::{ Backend, CrosstermBackend }, Terminal };
use std::{ error::Error, io::{ self, Stdout }, time::{ Duration, Instant } };
use tokio::{ io::{ AsyncBufReadExt, AsyncWriteExt, BufReader }, net::TcpStream, sync::mpsc };

mod ui;
mod app;
use app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server_addr = "10.16.4.22:56570";
    let stream = match TcpStream::connect(server_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Sunucuya bağlanılamadı: {}. Lütfen sunucunun çalıştığından emin olun.", e);
            return Ok(());
        }
    };
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let (ui_to_network_tx, mut ui_to_network_rx) = mpsc::channel::<String>(32);
    let (network_to_ui_tx, network_to_ui_rx) = mpsc::channel::<String>(32);

    //Read task*********************************************************************
    let network_tx_clone = network_to_ui_tx.clone();
    let read_task = tokio::spawn(async move {
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    let _ = network_tx_clone.send("Sunucu ile bağlantı kapandı.".to_string()).await;
                    break;
                }
                Ok(_) => {
                    if network_tx_clone.send(line.trim_end().to_string()).await.is_err() {
                        break;
                    }
                    line.clear();
                }
                Err(e) => {
                    let _ = network_tx_clone.send(
                        format!("Sunucudan gelen mesaj okunamadı: {}", e)
                    ).await;
                    break;
                }
            }
        }
    });

    //Write task********************************************************************
    let write_task = tokio::spawn(async move {
        while let Some(message) = ui_to_network_rx.recv().await {
            let message_to_send = format!("{}\n", message);
            if writer.write_all(message_to_send.as_bytes()).await.is_err() {
                eprintln!("Sunucuya mesaj gönderilemedi.");
                break;
            }
            if message.trim().eq_ignore_ascii_case("/quit") {
                break;
            }
        }
    });

    let mut terminal = setup_terminal()?;
    let app = App::new();

    // --- Ana Uygulama Döngüsü ---
    let res = run_app(&mut terminal, app, network_to_ui_rx, ui_to_network_tx).await; // await eklendi

    let _ = tokio::join!(read_task, write_task);

    // --- Temizlik ---
    restore_terminal(&mut terminal)?;

    if let Err(e) = res {
        println!("Uygulama hatası: {:?}", e);
    }

    Ok(())
}

// Ana uygulama döngüsünü çalıştıran fonksiyon
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut network_to_ui_rx: mpsc::Receiver<String>,
    ui_to_network_tx: mpsc::Sender<String>
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250); // Döngü sıklığı (olay bekleme süresi)
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        match network_to_ui_rx.try_recv() {
            Ok(message) => {
                app.add_message(message);
            }
            Err(_) => {
                break;
            }
        }

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(message_to_send) = app.handle_enter() {
                                if ui_to_network_tx.send(message_to_send.clone()).await.is_err() {
                                    app.should_quit = true;
                                }

                                if message_to_send.trim().eq_ignore_ascii_case("/quit") {
                                    app.should_quit = true;
                                }
                            }
                        }
                        KeyCode::Up => app.scroll_up(),

                        KeyCode::Down => app.scroll_down(),

                        KeyCode::Char(c) => {
                            app.input.push(c);
                            app.scroll_to_bottom();
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                            app.scroll_to_bottom();
                        }
                        KeyCode::Esc => {
                            // Esc ile çıkış
                            app.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            if !app.input.trim().eq_ignore_ascii_case("/quit") {
                let _ = ui_to_network_tx.try_send("/quit".to_string());
            }
            return Ok(()); // Döngüden çık
        }
    }
    Ok(())
}

// Terminali kuran yardımcı fonksiyon
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?; // Ham modu etkinleştir
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; // Alternatif ekrana geç, fare olaylarını yakala (isteğe bağlı)
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

// Terminali eski haline getiren yardımcı fonksiyon
fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?; // Ham modu devre dışı bırak
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture // Fare olaylarını bırak
    )?;
    terminal.show_cursor()?; // İmleci tekrar göster
    Ok(())
}
