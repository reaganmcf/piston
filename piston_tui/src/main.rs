use std::{
    io::stdout,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use piston_ipc::{
    messages::{IpcMessage, Ping, Pong},
    IpcReader,
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame, Terminal,
};

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let (tx, rx) = mpsc::channel::<IpcMessage>();
    thread::spawn(move || {
        socket_thread(tx).expect("Failed to start socket thread");
    });

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui(f, &rx))?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> std::io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn socket_thread(tx: Sender<IpcMessage>) -> std::io::Result<()> {
    let reader = IpcReader::new()?;
    let stream = reader.as_stream()?;

    for msg in stream {
        match msg {
            Ok(msg) => {
                tx.send(msg).expect("Failed to send message");
            }
            Err(e) => {
                println!("Failed to read message, {:?}", e);
            }
        }
    }

    Ok(())
}

fn ui(frame: &mut Frame, rx: &Receiver<IpcMessage>) {
    match rx.try_recv().ok() {
        Some(msg) => {
            let text = match msg {
                IpcMessage::PortfolioStats(stats) => {
                    format!(
                        "Portfolio: {}\nPositions: {}\nTrades: {}\nRealized PnL: {}\nUnrealized PnL: {}",
                        stats.code,
                        stats.positions.len(),
                        stats.trade_count,
                        stats.pnl,
                        stats.unrealized_pnl
                    )
                }
                IpcMessage::Ping(Ping) => "Ping".to_string(),
                IpcMessage::Pong(Pong) => "Pong".to_string(),
            };

            frame.render_widget(
                Paragraph::new(text).block(Block::default().title("Piston").borders(Borders::ALL)),
                frame.size(),
            );
        }
        _ => {}
    };
}
