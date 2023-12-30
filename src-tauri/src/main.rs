// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use tauri::Window;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use tokio::time::sleep;

mod terminal;
mod font;

struct State {
  term: Arc<Mutex<terminal::Terminal>>,
  reader: File,
}

#[tauri::command]
async fn write_to_pty(data: u8, state: tauri::State<'_, State>) -> Result<(), String> {
    let term = state.term.clone();
    term.lock().await.write_to_pty(data as char);
    Ok(())
}

#[tauri::command]
async fn resize_term(rows: u16, cols: u16, state: tauri::State<'_, State>) -> Result<(), String> {
    let term = state.term.clone();
    term.lock().await.resize(rows, cols);
    Ok(())
}

#[tauri::command]
async fn init_read(window: Window, state: tauri::State<'_, State>) -> Result<(), String> {
    let mut reader = state.reader.try_clone().await.unwrap();
    let term = state.term.clone();

    tokio::spawn(async move {
        loop {
            let mut buf = [0; 4096];
            if let Ok(_) = reader.read(&mut buf).await {
                let mut term = term.lock().await;
                term.update(buf.to_vec());
                let cells = term.cells();
                window.emit("term_data", cells).unwrap();
            };
            sleep(std::time::Duration::from_millis(1)).await;
        }
    });
    Ok(())
}

#[tauri::command]
async fn init_raw_read(window: Window, state: tauri::State<'_, State>) -> Result<(), String> {
    let mut reader = state.reader.try_clone().await.unwrap();

    tokio::spawn(async move {
        loop {
            let mut buf = [0; 4096];
            if let Ok(_) = reader.read(&mut buf).await {
                window.emit("term_data", buf.to_vec()).unwrap();
            };
            sleep(std::time::Duration::from_millis(1)).await;
        }
    });
    Ok(())
}

fn main() {
    let mut term = terminal::Terminal::new("/bin/bash".to_string());
    let reader = File::from(term.new_reader());

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![init_read, init_raw_read, write_to_pty, resize_term])
        .manage(State{
            term: Arc::new(Mutex::new(term)),
            reader,
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
