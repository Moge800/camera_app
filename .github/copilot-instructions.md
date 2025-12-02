# GitHub Copilot Instructions

## プロジェクト概要
Surface Go 4向けのカメラアプリケーション。OpenCVでカメラアクセスし、eframe/eGuiでGUIを構築。写真撮影、動画録画、カメラ切り替え機能を提供する。

## 技術スタック
- **言語**: Rust 2021 Edition
- **ビルドツール**: Cargo
- **GUI**: eframe 0.29, egui 0.29
- **カメラ/画像処理**: opencv-rust 0.92 (videoio, imgcodecs, imgproc)
- **日時処理**: chrono 0.4

## プロジェクト構造
```
src/
└── main.rs           # メインアプリケーション
target/               # ビルド成果物
camera_output/        # 写真・動画の保存先
Cargo.toml            # 依存関係定義
```

**主要コンポーネント**:
- `CameraApp`: アプリケーション本体
  - カメラ管理 (`VideoCapture`)
  - 動画録画 (`VideoWriter`)
  - フレーム更新・表示
  - UI制御

## コーディング規約

### 1. 所有権とライフタイム
```rust
// Good - Arc<Mutex>で安全に共有
camera: Arc<Mutex<Option<VideoCapture>>>,

// Bad - グローバル変数や生ポインタ
static mut CAMERA: Option<VideoCapture> = None;
```

### 2. エラーハンドリング
- `unwrap()`は避ける
- `if let Ok()`, `match`, `unwrap_or()`を使用
```rust
// Good
if let Ok(mut cam_lock) = self.camera.lock() {
    // 処理
}

// Bad
let cam = self.camera.lock().unwrap();
```

### 3. スレッドセーフティ
- 複数スレッドから参照される状態は`Arc<Mutex<T>>`または`Arc<AtomicBool>`
- `is_recording`は`Arc<AtomicBool>`でロックフリー実装
```rust
// Good
is_recording: Arc<AtomicBool>
// 読み取り
self.is_recording.load(Ordering::Relaxed)
// 書き込み
self.is_recording.store(true, Ordering::Relaxed)

// Bad
is_recording: bool  // &selfメソッドから変更不可
```

### 4. Mutexのロック管理
- ロックスコープは最小限に
- デッドロックを避けるため、複数Mutexの取得順序を統一
```rust
// Good
if let Ok(cam_lock) = self.camera.lock() {
    // 必要な処理のみ
}

// Bad
let cam_lock = self.camera.lock().unwrap(); // 関数全体でロック保持
```

### 5. OpenCVのリソース管理
- `Mat`は自動的にメモリ管理される
- `VideoCapture`, `VideoWriter`は`Drop`で自動クリーンアップ
- 手動でリソース解放が必要な場合は`drop()`を明示
```rust
drop(writer); // VideoWriterを即座に解放
```

### 6. ファイルパスの扱い
- `PathBuf`を使用
- `to_str()`でOption処理
```rust
// Good
let filename = self.output_dir.join(format!("photo_{}.jpg", timestamp));
filename.to_str().unwrap_or("photo.jpg")

// Bad
let filename = format!("camera_output/photo_{}.jpg", timestamp);
```

### 7. 定数とマジックナンバー
```rust
// Good
const DEFAULT_FPS: f64 = 30.0;
const MIN_FPS: f64 = 0.0;
const MAX_FPS: f64 = 120.0;

// Bad
let fps = if fps > 0.0 && fps <= 120.0 { fps } else { 30.0 };
```

### 8. importの順序
```rust
// 標準ライブラリ
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;
use std::fs;

// サードパーティクレート
use eframe::egui;
use opencv::prelude::*;
use chrono::Local;
```

### 9. 型の明示
```rust
// Good - 型が明確
fn init_camera(&mut self) -> Result<(), String>

// Bad - 推論に頼りすぎ
fn init_camera(&mut self)
```

### 10. UIロジックの分離
- ビジネスロジックとUI描画を分離
- eGUIの`update()`はUI描画に専念
- データ更新は専用メソッド(`update_frame()`, `capture_photo()`等)

## OpenCV使用時の注意点

### カメラアクセス
- `VideoCapture::new(index, CAP_ANY)`でカメラオープン
- `is_opened()`で開けたか確認
- 解像度設定は`CAP_PROP_FRAME_WIDTH/HEIGHT`
- 実際の解像度は`get()`で取得確認

### 動画録画
- `VideoWriter::fourcc()`でコーデック指定
- `mp4v`推奨(互換性高い)、フォールバックは`MJPG`
- FPS検証: 0以下または異常に高い値を弾く
- `is_opened()`で書き込み可能か確認

### 色空間変換
- OpenCVはBGR、eGuiはRGB
- `cvt_color()`で`COLOR_BGR2RGB`変換必須
```rust
opencv::imgproc::cvt_color(&frame, &mut rgb_frame, 
                           opencv::imgproc::COLOR_BGR2RGB, 0)
```

## eframe/eGui特有の考慮事項

### リアルタイム更新
- `ctx.request_repaint()`で継続的に再描画
- フレームレート制御は不要(eGuiが自動調整)

### テクスチャ管理
- `ctx.load_texture()`で画像をGPUにアップロード
- 同じ名前で上書き可能(自動置き換え)
```rust
let texture = ctx.load_texture(
    "camera_frame",
    frame.clone(),
    Default::default()
);
```

### ウィンドウサイズ
- `ViewportBuilder::with_inner_size()`で初期サイズ指定
- `ui.available_size()`で動的にサイズ取得

## セキュリティとプライバシー

### カメラアクセス権限
- Windows: 初回実行時に権限要求
- ユーザーが拒否した場合のエラーハンドリング必須

### ファイル保存
- `camera_output/`ディレクトリに集約
- タイムスタンプでファイル名生成(`YYYYMMDD_HHMMSS`)
- 上書き防止

## ビルドとデプロイ

### 開発ビルド
```powershell
cargo build
cargo run
```

### リリースビルド
```powershell
cargo build --release
cargo run --release
```

### OpenCVセットアップ(Windows)
```powershell
# vcpkg経由
vcpkg install opencv4[contrib,nonfree]:x64-windows

# 環境変数設定
$env:OPENCV_LINK_LIBS="opencv_world4"
$env:OPENCV_LINK_PATHS="C:\vcpkg\installed\x64-windows\lib"
$env:OPENCV_INCLUDE_PATHS="C:\vcpkg\installed\x64-windows\include"
```

## よくある問題と解決策

### カメラが開けない
- カメラインデックスを確認(0, 1, 2...)
- 他のアプリがカメラを使用していないか
- カメラ権限設定を確認

### 録画ファイルが再生できない
- コーデックを`MJPG`に変更
- FPSが正常な値か確認
- ファイルが完全に閉じられているか(`drop(writer)`)

### フレームレートが低い
- カメラの解像度を下げる(640x480推奨)
- リリースビルドで実行(`--release`)
- RGB変換処理の最適化

### Mutexデッドロック
- ロック取得順序を統一
- ロックスコープを最小化
- `if let Ok()`で失敗時の処理を明示

## コード品質

### Linter/Formatter
```powershell
# フォーマット
cargo fmt

# Lint
cargo clippy -- -D warnings

# テスト
cargo test
```

### 推奨Clippy設定
```toml
# Cargo.toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "warn"
```

## 命名規則
- 構造体/Enum: `PascalCase` (例: `CameraApp`, `CaptureMode`)
- 関数/変数: `snake_case` (例: `init_camera`, `frame_width`)
- 定数: `UPPER_SNAKE_CASE` (例: `DEFAULT_FPS`)

## ドキュメント
```rust
/// カメラを初期化し、指定された解像度で開く
///
/// # 戻り値
/// 成功時は`Ok(())`、失敗時はエラーメッセージを含む`Err(String)`
fn init_camera(&mut self) -> Result<(), String> {
    // ...
}
```

## テスト戦略

### 単体テスト
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_switch_logic() {
        let mut app = CameraApp::default();
        assert_eq!(app.camera_index, 0);
        app.camera_index = if app.camera_index == 0 { 1 } else { 0 };
        assert_eq!(app.camera_index, 1);
    }
}
```

### 統合テスト
- OpenCVモックは難しいため、手動テスト推奨
- カメラ接続・切り替え・撮影・録画の動作確認

## Git管理

### .gitignore対象
```
/target/
/camera_output/
Cargo.lock  # ライブラリの場合
*.exe
*.dll
```

### コミットメッセージ
```
feat: 動画録画機能を追加
fix: カメラ切り替え時のデッドロックを修正
refactor: エラーハンドリングをif letに統一
docs: READMEにOpenCVセットアップ手順を追加
```

## パフォーマンス最適化

### リリースビルド最適化
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### メモリ使用量削減
- フレームバッファは1つに制限
- 不要な`clone()`を避ける
- `Arc`で共有、`Mutex`でロック

## 今後の拡張案
- [ ] 設定UIの追加(解像度、FPS、保存先)
- [ ] サムネイル一覧表示
- [ ] フィルター機能
- [ ] クラウドアップロード
- [ ] マルチカメラ同時録画

---

**このプロジェクトは個人開発プロジェクトです。改善提案や質問は歓迎します！**
