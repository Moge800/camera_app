# Camera App for Surface Go 4

Surface Go 4向けに最適化されたカメラアプリケーション。OpenCVとeGuiを使用して、写真撮影、動画録画、カメラ切り替え機能を提供します。

## 特徴

- 📷 **写真撮影** - 高画質のJPEG形式で保存
- 🎥 **動画録画** - MP4形式での録画に対応
- 🔄 **カメラ切り替え** - リアカメラとフロントカメラの簡単切り替え
- 🖥️ **リアルタイムプレビュー** - スムーズなカメラ映像表示
- 💾 **自動保存** - タイムスタンプ付きファイル名で整理

## 必要要件

### システム要件
- Windows 10/11 (Surface Go 4推奨)
- カメラデバイス(内蔵または外付け)

### 開発環境
- Rust 1.70以上
- Cargo
- OpenCV 4.x

## セットアップ

### 1. OpenCVとLLVMのインストール

#### vcpkg使用(推奨)

```powershell
# vcpkgのインストール (未インストールの場合)
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# LLVM/Clangのインストール (opencv-rustのビルドに必要)
.\vcpkg install llvm:x64-windows

# OpenCVのインストール
.\vcpkg install opencv4[contrib,nonfree]:x64-windows

# 環境変数の設定 (現在のセッションのみ)
$env:OPENCV_LINK_LIBS="opencv_world4"
$env:OPENCV_LINK_PATHS="C:\vcpkg\installed\x64-windows\lib"
$env:OPENCV_INCLUDE_PATHS="C:\vcpkg\installed\x64-windows\include"
$env:LIBCLANG_PATH="C:\vcpkg\installed\x64-windows\bin"

# 環境変数を永続的に設定 (推奨)
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_LIBS", "opencv_world4", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_PATHS", "C:\vcpkg\installed\x64-windows\lib", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_INCLUDE_PATHS", "C:\vcpkg\installed\x64-windows\include", "User")
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\vcpkg\installed\x64-windows\bin", "User")

# 設定後はPowerShellを再起動してください
```

#### 手動インストール

1. [LLVM公式サイト](https://releases.llvm.org/)からLLVMをダウンロード・インストール
2. [OpenCV公式サイト](https://opencv.org/releases/)からWindows版をダウンロード
3. 任意の場所に解凍
4. 環境変数を設定:
   ```powershell
   $env:OPENCV_LINK_LIBS="opencv_world4"
   $env:OPENCV_LINK_PATHS="C:\opencv\build\x64\vc16\lib"
   $env:OPENCV_INCLUDE_PATHS="C:\opencv\build\include"
   $env:LIBCLANG_PATH="C:\Program Files\LLVM\bin"
   ```

### 2. プロジェクトのビルド

```powershell
# リポジトリのクローン
git clone https://github.com/Moge800/camera_app.git
cd camera_app

# 開発ビルド
cargo build

# リリースビルド (推奨)
cargo build --release
```

## 使い方

### アプリの起動

```powershell
# 開発ビルドの実行
cargo run

# リリースビルドの実行
cargo run --release

# またはバイナリを直接実行
.\target\release\camera_app.exe
```

### 基本操作

1. **モード切り替え**
   - 📷 写真モード: 静止画撮影
   - 🎥 動画モード: ビデオ録画

2. **カメラ切り替え**
   - 🔲 リア: 背面カメラ (デフォルト: カメラインデックス0)
   - 🤳 フロント: 前面カメラ (カメラインデックス1)

3. **撮影**
   - 写真モード: 「📸 写真を撮る」ボタンをクリック
   - 動画モード: 「⏺ 録画開始」→「⏹ 録画停止」

### ファイルの保存先

すべての写真と動画は `camera_output/` ディレクトリに保存されます:

- 写真: `photo_YYYYMMDD_HHMMSS.jpg`
- 動画: `video_YYYYMMDD_HHMMSS.mp4`

## 開発

### コードフォーマット

```powershell
cargo fmt
```

### Lint

```powershell
cargo clippy -- -D warnings
```

### テスト

```powershell
cargo test
```

## トラブルシューティング

### カメラが開けない

- カメラのプライバシー設定を確認してください
- 他のアプリケーションがカメラを使用していないか確認してください
- カメラインデックス(0, 1, 2...)を試してください

### 録画ファイルが再生できない

- コーデックの問題の可能性があります
- VLC Media Playerなど、対応プレーヤーで再生を試してください
- 録画を完全に停止してからファイルを開いてください

### フレームレートが低い

- リリースビルドで実行してください (`cargo run --release`)
- カメラの解像度設定を下げてください (デフォルト: 640x480)

## ライセンス

MIT License - 詳細は[LICENSE](LICENSE)ファイルを参照してください。

## 貢献

プルリクエストや改善提案を歓迎します！

1. このリポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'feat: Add amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## 作者

Moge800

## 謝辞

- [opencv-rust](https://github.com/twistedfall/opencv-rust) - OpenCVのRustバインディング
- [egui](https://github.com/emilk/egui) - 即座に動作するGUIライブラリ
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - eGuiのフレームワーク
