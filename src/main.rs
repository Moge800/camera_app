use eframe::egui;
use opencv::{
    core::{Mat, Size, Vector},
    imgcodecs,
    prelude::*,
    videoio::{self, VideoCapture, VideoWriter},
};
use chrono::Local;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;
use std::fs;

#[derive(PartialEq, Clone, Copy)]
enum CaptureMode {
    Photo,
    Video,
}

#[derive(PartialEq, Clone, Copy)]
enum CameraPosition {
    Front,
    Rear,
}

struct CameraApp {
    camera: Arc<Mutex<Option<VideoCapture>>>,
    video_writer: Arc<Mutex<Option<VideoWriter>>>,
    current_frame: Arc<Mutex<Option<egui::ColorImage>>>,
    capture_mode: CaptureMode,
    camera_position: CameraPosition,
    is_recording: Arc<AtomicBool>,
    camera_index: i32,
    frame_width: i32,
    frame_height: i32,
    output_dir: PathBuf,
}

impl Default for CameraApp {
    fn default() -> Self {
        // 出力ディレクトリを作成
        let output_dir = PathBuf::from("camera_output");
        if !output_dir.exists() {
            let _ = fs::create_dir_all(&output_dir);
        }
        
        Self {
            camera: Arc::new(Mutex::new(None)),
            video_writer: Arc::new(Mutex::new(None)),
            current_frame: Arc::new(Mutex::new(None)),
            capture_mode: CaptureMode::Photo,
            camera_position: CameraPosition::Rear,
            is_recording: Arc::new(AtomicBool::new(false)),
            camera_index: 0,
            frame_width: 640,
            frame_height: 480,
            output_dir,
        }
    }
}

impl CameraApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        app.init_camera();
        app
    }

    fn init_camera(&mut self) {
        match VideoCapture::new(self.camera_index, videoio::CAP_ANY) {
            Ok(mut cam) => {
                if cam.is_opened().unwrap_or(false) {
                    // カメラの解像度を設定
                    let _ = cam.set(videoio::CAP_PROP_FRAME_WIDTH, self.frame_width as f64);
                    let _ = cam.set(videoio::CAP_PROP_FRAME_HEIGHT, self.frame_height as f64);
                    
                    // 実際の解像度を取得
                    if let Ok(width) = cam.get(videoio::CAP_PROP_FRAME_WIDTH) {
                        self.frame_width = width as i32;
                    }
                    if let Ok(height) = cam.get(videoio::CAP_PROP_FRAME_HEIGHT) {
                        self.frame_height = height as i32;
                    }
                    
                    *self.camera.lock().unwrap() = Some(cam);
                    println!("カメラを初期化しました ({}x{})", self.frame_width, self.frame_height);
                } else {
                    eprintln!("カメラを開けませんでした");
                }
            }
            Err(e) => {
                eprintln!("カメラの初期化に失敗しました: {}", e);
            }
        }
    }

    fn switch_camera(&mut self) {
        // 録画中の場合は停止
        if self.is_recording.load(Ordering::Relaxed) {
            self.stop_recording();
        }
        
        // 現在のカメラを解放
        if let Ok(mut cam_lock) = self.camera.lock() {
            *cam_lock = None;
        }
        
        // カメラインデックスを切り替え (0: リア, 1: フロント)
        self.camera_index = if self.camera_index == 0 { 1 } else { 0 };
        
        // カメラを再初期化
        self.init_camera();
    }

    fn capture_photo(&self) {
        if let Ok(mut cam_lock) = self.camera.lock() {
            if let Some(cam) = cam_lock.as_mut() {
                let mut frame = Mat::default();
                if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
                    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                    let filename = self.output_dir.join(format!("photo_{}.jpg", timestamp));
                    
                    match imgcodecs::imwrite(filename.to_str().unwrap_or("photo.jpg"), &frame, &Vector::new()) {
                        Ok(_) => println!("写真を保存しました: {:?}", filename),
                        Err(e) => eprintln!("写真の保存に失敗しました: {}", e),
                    }
                }
            }
        }
    }

    fn start_recording(&mut self) {
        if let Ok(cam_lock) = self.camera.lock() {
            if let Some(cam) = cam_lock.as_ref() {
                let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                let filename = self.output_dir.join(format!("video_{}.mp4", timestamp));
                
                // MP4形式で保存 (H264コーデック)
                let fourcc = VideoWriter::fourcc('m', 'p', '4', 'v').unwrap_or(
                    VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap_or(0)
                );
                
                // FPSを取得、不正な値の場合は30fpsをデフォルトに
                let fps = cam.get(videoio::CAP_PROP_FPS).unwrap_or(30.0);
                let fps = if fps > 0.0 && fps <= 120.0 { fps } else { 30.0 };
                let frame_size = Size::new(self.frame_width, self.frame_height);
                
                match VideoWriter::new(filename.to_str().unwrap_or("video.mp4"), fourcc, fps, frame_size, true) {
                    Ok(writer) => {
                        if writer.is_opened().unwrap_or(false) {
                            if let Ok(mut writer_lock) = self.video_writer.lock() {
                                *writer_lock = Some(writer);
                                self.is_recording.store(true, Ordering::Relaxed);
                                println!("録画を開始しました: {:?} ({}fps)", filename, fps);
                            }
                        } else {
                            eprintln!("VideoWriterを開けませんでした");
                        }
                    }
                    Err(e) => {
                        eprintln!("VideoWriterの作成に失敗しました: {}", e);
                    }
                }
            }
        }
    }

    fn stop_recording(&mut self) {
        if let Ok(mut writer_lock) = self.video_writer.lock() {
            if let Some(writer) = writer_lock.take() {
                drop(writer);
                self.is_recording.store(false, Ordering::Relaxed);
                println!("録画を停止しました");
            }
        }
    }

    fn update_frame(&self) {
        if let Ok(mut cam_lock) = self.camera.lock() {
            if let Some(cam) = cam_lock.as_mut() {
                let mut frame = Mat::default();
                
                if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
                    // 録画中の場合はフレームを書き込む
                    if self.is_recording.load(Ordering::Relaxed) {
                        if let Ok(mut writer_lock) = self.video_writer.lock() {
                            if let Some(writer) = writer_lock.as_mut() {
                                let _ = writer.write(&frame);
                            }
                        }
                    }
                    
                    // フレームをRGBに変換
                    let mut rgb_frame = Mat::default();
                    if opencv::imgproc::cvt_color(&frame, &mut rgb_frame, opencv::imgproc::COLOR_BGR2RGB, 0).is_ok() {
                        if let Ok(size) = rgb_frame.size() {
                            let width = size.width as usize;
                            let height = size.height as usize;
                            
                            if let Ok(data) = rgb_frame.data_bytes() {
                                let pixels: Vec<egui::Color32> = data
                                    .chunks(3)
                                    .map(|rgb| egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
                                    .collect();
                                
                                if pixels.len() == width * height {
                                    let color_image = egui::ColorImage {
                                        size: [width, height],
                                        pixels,
                                    };
                                    
                                    if let Ok(mut frame_lock) = self.current_frame.lock() {
                                        *frame_lock = Some(color_image);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for CameraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // フレームを更新
        self.update_frame();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Surface Go 4 カメラアプリ (OpenCV)");
            
            ui.separator();
            
            // カメラプレビュー表示
            if let Some(frame) = self.current_frame.lock().unwrap().as_ref() {
                let texture = ctx.load_texture(
                    "camera_frame",
                    frame.clone(),
                    Default::default()
                );
                
                let available_size = ui.available_size();
                let image_size = [
                    available_size.x.min(800.0),
                    available_size.y - 150.0,
                ];
                
                ui.add(
                    egui::Image::new(&texture)
                        .fit_to_exact_size(egui::vec2(image_size[0], image_size[1]))
                );
            } else {
                ui.label("カメラを初期化中...");
            }
            
            ui.separator();
            
            // コントロールパネル
            ui.horizontal(|ui| {
                // モード切り替えトグル
                ui.label("モード:");
                if ui.selectable_label(
                    self.capture_mode == CaptureMode::Photo,
                    "📷 写真"
                ).clicked() {
                    if self.is_recording.load(Ordering::Relaxed) {
                        self.stop_recording();
                    }
                    self.capture_mode = CaptureMode::Photo;
                }
                
                if ui.selectable_label(
                    self.capture_mode == CaptureMode::Video,
                    "🎥 動画"
                ).clicked() {
                    self.capture_mode = CaptureMode::Video;
                }
                
                ui.separator();
                
                // カメラ切り替えトグル
                ui.label("カメラ:");
                if ui.selectable_label(
                    self.camera_position == CameraPosition::Rear,
                    "🔲 リア"
                ).clicked() {
                    if self.camera_position != CameraPosition::Rear {
                        self.camera_position = CameraPosition::Rear;
                        self.switch_camera();
                    }
                }
                
                if ui.selectable_label(
                    self.camera_position == CameraPosition::Front,
                    "🤳 フロント"
                ).clicked() {
                    if self.camera_position != CameraPosition::Front {
                        self.camera_position = CameraPosition::Front;
                        self.switch_camera();
                    }
                }
            });
            
            ui.separator();
            
            // 撮影ボタン
            ui.horizontal(|ui| {
                match self.capture_mode {
                    CaptureMode::Photo => {
                        if ui.button("📸 写真を撮る").clicked() {
                            self.capture_photo();
                        }
                    }
                    CaptureMode::Video => {
                        if !self.is_recording.load(Ordering::Relaxed) {
                            if ui.button("⏺ 録画開始").clicked() {
                                self.start_recording();
                            }
                        } else {
                            if ui.button("⏹ 録画停止").clicked() {
                                self.stop_recording();
                            }
                            ui.label("🔴 録画中...");
                        }
                    }
                }
            });
            
            ui.separator();
            ui.label(format!("保存先: {}", self.output_dir.display()));
        });
        
        // 継続的に再描画
        ctx.request_repaint();
    }
}

impl Drop for CameraApp {
    fn drop(&mut self) {
        // 録画中の場合は停止
        if self.is_recording.load(Ordering::Relaxed) {
            self.stop_recording();
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Surface Go 4 カメラアプリ"),
        ..Default::default()
    };
    
    eframe::run_native(
        "camera_app",
        options,
        Box::new(|cc| Ok(Box::new(CameraApp::new(cc)))),
    )
}
