// 標準ライブラリのインポート
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;
use std::fs;

// サードパーティクレートのインポート
use eframe::egui;
use opencv::{
    core::{Mat, Size, Vector},
    imgcodecs,
    prelude::*,
    videoio::{self, VideoCapture, VideoWriter},
};
use chrono::Local;

/// キャプチャモード: 写真撮影か動画録画かを区別
#[derive(PartialEq, Clone, Copy)]
enum CaptureMode {
    Photo,  // 写真撮影モード
    Video,  // 動画録画モード
}

/// カメラポジション: フロントカメラかリアカメラかを区別
#[derive(PartialEq, Clone, Copy)]
enum CameraPosition {
    Front,  // フロントカメラ
    Rear,   // リアカメラ
}

/// カメラアプリケーションのメイン構造体
///
/// OpenCVを使用したカメラアクセスと、eGuiを使用したUI表示を統合する。
/// スレッドセーフな設計により、バックグラウンドでのフレーム更新と録画を実現。
struct CameraApp {
    /// カメラデバイス (複数スレッドからアクセス可能にするためArc<Mutex>で保護)
    camera: Arc<Mutex<Option<VideoCapture>>>,
    /// 動画書き込み用 (録画中のみ使用)
    video_writer: Arc<Mutex<Option<VideoWriter>>>,
    /// 現在のカメラフレーム (eGui描画用に変換済み)
    current_frame: Arc<Mutex<Option<egui::ColorImage>>>,
    /// 現在のキャプチャモード (写真/動画)
    capture_mode: CaptureMode,
    /// 現在のカメラポジション (フロント/リア)
    camera_position: CameraPosition,
    /// 録画中かどうか (ロックフリーなアトミック変数で管理)
    is_recording: Arc<AtomicBool>,
    /// カメラデバイスのインデックス (0: リア, 1: フロント)
    camera_index: i32,
    /// フレームの幅 (ピクセル)
    frame_width: i32,
    /// フレームの高さ (ピクセル)
    frame_height: i32,
    /// 写真・動画の保存先ディレクトリ
    output_dir: PathBuf,
}

impl Default for CameraApp {
    /// デフォルトのアプリケーション設定を構築
    ///
    /// 初期状態として、リアカメラ、写真モード、640x480の解像度を設定。
    /// 出力ディレクトリ (camera_output/) が存在しない場合は作成する。
    fn default() -> Self {
        // 出力ディレクトリを作成 (存在しない場合のみ)
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
            camera_index: 0,  // 0: リアカメラ (デフォルト)
            frame_width: 640,  // 640x480は互換性が高い
            frame_height: 480,
            output_dir,
        }
    }
}

impl CameraApp {
    /// eframe起動時に呼ばれる初期化関数
    ///
    /// デフォルト設定でアプリケーションを構築し、カメラを初期化する。
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        app.init_camera();
        app
    }

    /// カメラデバイスを初期化
    ///
    /// 指定されたカメラインデックスでVideoCaptureを開き、解像度を設定する。
    /// 設定した解像度が実際に適用されたかを確認し、実際の値を保存する。
    fn init_camera(&mut self) {
        match VideoCapture::new(self.camera_index, videoio::CAP_ANY) {
            Ok(mut cam) => {
                if cam.is_opened().unwrap_or(false) {
                    // カメラの解像度を設定 (リクエスト)
                    let _ = cam.set(videoio::CAP_PROP_FRAME_WIDTH, self.frame_width as f64);
                    let _ = cam.set(videoio::CAP_PROP_FRAME_HEIGHT, self.frame_height as f64);

                    // 実際に設定された解像度を取得 (デバイスによっては異なる場合がある)
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

    /// カメラを切り替える (フロント ⇔ リア)
    ///
    /// 録画中の場合は先に停止し、現在のカメラを解放してから
    /// カメラインデックスを切り替えて再初期化する。
    fn switch_camera(&mut self) {
        // 録画中の場合は停止 (カメラ切り替え時に録画を継続できないため)
        if self.is_recording.load(Ordering::Relaxed) {
            self.stop_recording();
        }

        // 現在のカメラを解放 (Mutexロックを取得してNoneに設定)
        if let Ok(mut cam_lock) = self.camera.lock() {
            *cam_lock = None;
        }

        // カメラインデックスを切り替え (0 ⇔ 1)
        // 0: リアカメラ, 1: フロントカメラ (一般的な配置)
        self.camera_index = if self.camera_index == 0 { 1 } else { 0 };

        // 新しいカメラインデックスで再初期化
        self.init_camera();
    }

    /// 写真を撮影して保存
    ///
    /// カメラから1フレームを読み取り、タイムスタンプ付きのファイル名でJPEG形式で保存。
    /// ファイル名形式: photo_YYYYMMDD_HHMMSS.jpg
    fn capture_photo(&self) {
        // カメラのMutexロックを取得
        if let Ok(mut cam_lock) = self.camera.lock() {
            if let Some(cam) = cam_lock.as_mut() {
                let mut frame = Mat::default();
                // カメラから1フレーム読み取り
                if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
                    // タイムスタンプでファイル名を生成 (重複を防ぐ)
                    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                    let filename = self.output_dir.join(format!("photo_{}.jpg", timestamp));

                    // JPEG形式で保存 (OpenCVのimwrite関数)
                    match imgcodecs::imwrite(filename.to_str().unwrap_or("photo.jpg"), &frame, &Vector::new()) {
                        Ok(_) => println!("写真を保存しました: {:?}", filename),
                        Err(e) => eprintln!("写真の保存に失敗しました: {}", e),
                    }
                }
            }
        }
    }

    /// 動画録画を開始
    ///
    /// VideoWriterを作成し、MP4形式で録画を開始する。
    /// コーデックはmp4v (H264互換)を試み、失敗時はMJPGにフォールバック。
    /// FPSはカメラから取得し、不正な値の場合は30fpsをデフォルトとする。
    fn start_recording(&mut self) {
        // カメラのMutexロックを取得 (読み取り専用)
        if let Ok(cam_lock) = self.camera.lock() {
            if let Some(cam) = cam_lock.as_ref() {
                // タイムスタンプでファイル名を生成
                let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                let filename = self.output_dir.join(format!("video_{}.mp4", timestamp));

                // MP4形式で保存 (H264コーデック)
                // fourcc: Four Character Code (動画コーデック識別子)
                // mp4v: MPEG-4 Part 2 (互換性が高い)
                // MJPG: Motion JPEG (フォールバック用)
                let fourcc = VideoWriter::fourcc('m', 'p', '4', 'v').unwrap_or(
                    VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap_or(0)
                );

                // カメラのFPSを取得 (不正な値の場合は30fpsをデフォルト)
                let fps = cam.get(videoio::CAP_PROP_FPS).unwrap_or(30.0);
                let fps = if fps > 0.0 && fps <= 120.0 { fps } else { 30.0 };
                let frame_size = Size::new(self.frame_width, self.frame_height);

                // VideoWriterを作成
                match VideoWriter::new(filename.to_str().unwrap_or("video.mp4"), fourcc, fps, frame_size, true) {
                    Ok(writer) => {
                        // VideoWriterが正常に開けたか確認
                        if writer.is_opened().unwrap_or(false) {
                            // video_writerにVideoWriterを設定
                            if let Ok(mut writer_lock) = self.video_writer.lock() {
                                *writer_lock = Some(writer);
                                // 録画中フラグを立てる (アトミック操作)
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

    /// 動画録画を停止
    ///
    /// VideoWriterを解放し、ファイルをクローズする。
    /// drop()を明示的に呼ぶことで即座にリソースを解放する。
    fn stop_recording(&mut self) {
        // video_writerのMutexロックを取得
        if let Ok(mut writer_lock) = self.video_writer.lock() {
            // VideoWriterを取り出す (takeでOptionからSomeを取得、Noneに置き換え)
            if let Some(writer) = writer_lock.take() {
                // VideoWriterを即座に解放 (ファイルをクローズ)
                drop(writer);
                // 録画中フラグを下ろす (アトミック操作)
                self.is_recording.store(false, Ordering::Relaxed);
                println!("録画を停止しました");
            }
        }
    }

    /// カメラフレームを更新し、eGui用に変換
    ///
    /// カメラから1フレームを読み取り、以下の処理を行う:
    /// 1. 録画中の場合はVideoWriterにフレームを書き込む
    /// 2. BGR (OpenCV) → RGB (eGui) の色空間変換
    /// 3. バイトデータをegui::ColorImageに変換
    /// 4. current_frameに格納してUI表示用に提供
    fn update_frame(&self) {
        // カメラのMutexロックを取得
        if let Ok(mut cam_lock) = self.camera.lock() {
            if let Some(cam) = cam_lock.as_mut() {
                let mut frame = Mat::default();

                // カメラから1フレーム読み取り
                if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
                    // 録画中の場合はVideoWriterにフレームを書き込む
                    if self.is_recording.load(Ordering::Relaxed) {
                        if let Ok(mut writer_lock) = self.video_writer.lock() {
                            if let Some(writer) = writer_lock.as_mut() {
                                let _ = writer.write(&frame);
                            }
                        }
                    }

                    // フレームをBGR (OpenCV形式) からRGB (eGui形式) に変換
                    let mut rgb_frame = Mat::default();
                    if opencv::imgproc::cvt_color(&frame, &mut rgb_frame, opencv::imgproc::COLOR_BGR2RGB, 0).is_ok() {
                        // フレームのサイズを取得
                        if let Ok(size) = rgb_frame.size() {
                            let width = size.width as usize;
                            let height = size.height as usize;

                            // フレームのバイトデータを取得
                            if let Ok(data) = rgb_frame.data_bytes() {
                                // バイトデータをegui::Color32に変換
                                // 3バイト (R, G, B) を1ピクセルとして処理
                                let pixels: Vec<egui::Color32> = data
                                    .chunks(3)
                                    .map(|rgb| egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
                                    .collect();

                                // ピクセル数が正しいか確認 (width × height)
                                if pixels.len() == width * height {
                                    // egui::ColorImageを作成
                                    let color_image = egui::ColorImage {
                                        size: [width, height],
                                        pixels,
                                    };

                                    // current_frameに格納 (UI表示用)
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

/// eframe::Appトレイトの実装
///
/// eGuiのメインループで呼ばれるupdate関数を実装し、UIを描画する。
impl eframe::App for CameraApp {
    /// UIの更新と描画 (eGuiのメインループで毎フレーム呼ばれる)
    ///
    /// カメラフレームを更新し、UI要素 (プレビュー、モード切り替え、撮影ボタン等) を描画。
    /// ctx.request_repaint()で継続的に再描画を要求し、リアルタイム更新を実現。
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // カメラフレームを更新 (毎フレーム呼ばれる)
        self.update_frame();

        // 中央パネルを作成 (メインUI領域)
        egui::CentralPanel::default().show(ctx, |ui| {
            // アプリケーションタイトル
            ui.heading("Surface Go 4 カメラアプリ (OpenCV)");

            ui.separator();

            // カメラプレビュー表示
            if let Some(frame) = self.current_frame.lock().unwrap().as_ref() {
                // フレームをテクスチャとしてGPUにアップロード
                // 同じ名前 ("camera_frame") で上書きすることで自動的に更新される
                let texture = ctx.load_texture(
                    "camera_frame",
                    frame.clone(),
                    Default::default()
                );

                // 利用可能な画面サイズを取得
                let available_size = ui.available_size();
                // 画像表示サイズを計算 (最大800px幅、下部コントロール用に150px確保)
                let image_size = [
                    available_size.x.min(800.0),
                    available_size.y - 150.0,
                ];

                // 画像を表示 (指定サイズにフィット)
                ui.add(
                    egui::Image::new(&texture)
                        .fit_to_exact_size(egui::vec2(image_size[0], image_size[1]))
                );
            } else {
                // カメラ初期化中はメッセージを表示
                ui.label("カメラを初期化中...");
            }

            ui.separator();

            // コントロールパネル (モード切り替えとカメラ切り替え)
            ui.horizontal(|ui| {
                // キャプチャモード切り替えトグル (写真 or 動画)
                ui.label("モード:");
                // 写真モードボタン (選択中の場合ハイライト表示)
                if ui.selectable_label(
                    self.capture_mode == CaptureMode::Photo,
                    "📷 写真"
                ).clicked() {
                    // 録画中の場合は停止してから写真モードに切り替え
                    if self.is_recording.load(Ordering::Relaxed) {
                        self.stop_recording();
                    }
                    self.capture_mode = CaptureMode::Photo;
                }

                // 動画モードボタン (選択中の場合ハイライト表示)
                if ui.selectable_label(
                    self.capture_mode == CaptureMode::Video,
                    "🎥 動画"
                ).clicked() {
                    self.capture_mode = CaptureMode::Video;
                }

                ui.separator();

                // カメラ位置切り替えトグル (リア or フロント)
                ui.label("カメラ:");
                // リアカメラボタン (選択中の場合ハイライト表示)
                if ui.selectable_label(
                    self.camera_position == CameraPosition::Rear,
                    "🔲 リア"
                ).clicked() {
                    // 現在フロントカメラの場合のみ切り替え
                    if self.camera_position != CameraPosition::Rear {
                        self.camera_position = CameraPosition::Rear;
                        self.switch_camera();
                    }
                }

                // フロントカメラボタン (選択中の場合ハイライト表示)
                if ui.selectable_label(
                    self.camera_position == CameraPosition::Front,
                    "🤳 フロント"
                ).clicked() {
                    // 現在リアカメラの場合のみ切り替え
                    if self.camera_position != CameraPosition::Front {
                        self.camera_position = CameraPosition::Front;
                        self.switch_camera();
                    }
                }
            });

            ui.separator();

            // 撮影・録画ボタン (モードに応じて表示を切り替え)
            ui.horizontal(|ui| {
                match self.capture_mode {
                    CaptureMode::Photo => {
                        // 写真モード: 撮影ボタンを表示
                        if ui.button("📸 写真を撮る").clicked() {
                            self.capture_photo();
                        }
                    }
                    CaptureMode::Video => {
                        // 動画モード: 録画中かどうかで表示を切り替え
                        if !self.is_recording.load(Ordering::Relaxed) {
                            // 録画停止中: 録画開始ボタンを表示
                            if ui.button("⏺ 録画開始").clicked() {
                                self.start_recording();
                            }
                        } else {
                            // 録画中: 録画停止ボタンとステータス表示
                            if ui.button("⏹ 録画停止").clicked() {
                                self.stop_recording();
                            }
                            ui.label("🔴 録画中...");
                        }
                    }
                }
            });

            ui.separator();
            // 保存先ディレクトリを表示
            ui.label(format!("保存先: {}", self.output_dir.display()));
        });

        // 継続的に再描画を要求 (リアルタイム更新のため)
        ctx.request_repaint();
    }
}

/// Dropトレイトの実装
///
/// アプリケーション終了時にリソースをクリーンアップする。
/// 録画中の場合は自動的に停止し、VideoWriterを正常にクローズする。
impl Drop for CameraApp {
    fn drop(&mut self) {
        // 録画中の場合は停止 (ファイルを正常にクローズするため)
        if self.is_recording.load(Ordering::Relaxed) {
            self.stop_recording();
        }
    }
}

/// メイン関数: アプリケーションのエントリーポイント
///
/// eframeを起動し、CameraAppを実行する。
fn main() -> Result<(), eframe::Error> {
    // eframeのオプション設定
    let options = eframe::NativeOptions {
        // ビューポート (ウィンドウ) の設定
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])  // 初期ウィンドウサイズ
            .with_title("Surface Go 4 カメラアプリ"),  // ウィンドウタイトル
        ..Default::default()
    };

    // eframeを起動 (CameraAppを実行)
    eframe::run_native(
        "camera_app",
        options,
        Box::new(|cc| Ok(Box::new(CameraApp::new(cc)))),
    )
}
