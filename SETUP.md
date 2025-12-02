# OpenCVセットアップガイド (Windows)

このドキュメントでは、Windows環境でOpenCVとRustのopencv-rustクレートをセットアップする手順を説明します。

## 🚨 エラー内容

以下のようなエラーが発生した場合、このガイドに従ってセットアップしてください:

```
error: failed to run custom build command for `clang-sys v1.8.1`
couldn't find any valid shared libraries matching: ['clang.dll', 'libclang.dll']
set the `LIBCLANG_PATH` environment variable
```

このエラーは、opencv-rustのビルドに必要なLLVM/Clangが見つからないことを示しています。

---

## ✅ 解決方法: vcpkgを使用したセットアップ (推奨)

### ステップ1: vcpkgのインストール

vcpkgは、Microsoftが提供するC/C++パッケージマネージャーです。

```powershell
# vcpkgをクローン (C:\vcpkgにインストール)
git clone https://github.com/microsoft/vcpkg.git C:\vcpkg

# vcpkgディレクトリに移動
cd C:\vcpkg

# vcpkgをビルド (初回のみ)
.\bootstrap-vcpkg.bat
```

**所要時間**: 約3-5分

---

### ステップ2: LLVMのインストール

opencv-rustのビルドにはLLVM/Clangが必要です。

```powershell
# vcpkgでLLVMをインストール
cd C:\vcpkg
.\vcpkg install llvm:x64-windows
```

**所要時間**: 約30-60分 (初回のみ、ダウンロード速度により変動)

---

### ステップ3: OpenCVのインストール

```powershell
# vcpkgでOpenCV 4をインストール
cd C:\vcpkg
.\vcpkg install opencv4[contrib,nonfree]:x64-windows
```

**所要時間**: 約20-40分 (初回のみ、ダウンロード速度により変動)

---

### ステップ4: 環境変数の設定

#### オプションA: 永続的に設定 (推奨)

PowerShellで以下を実行 (管理者権限不要):

```powershell
# OPENCV関連の環境変数
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_LIBS", "opencv_world4", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_PATHS", "C:\vcpkg\installed\x64-windows\lib", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_INCLUDE_PATHS", "C:\vcpkg\installed\x64-windows\include", "User")

# LLVM/Clang関連の環境変数
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\vcpkg\installed\x64-windows\bin", "User")
[System.Environment]::SetEnvironmentVariable("LLVM_CONFIG_PATH", "C:\vcpkg\installed\x64-windows\tools\llvm\llvm-config.exe", "User")
```

**重要**: 設定後、PowerShellまたはVS Codeを再起動してください。

#### オプションB: 一時的に設定 (現在のセッションのみ)

```powershell
$env:OPENCV_LINK_LIBS="opencv_world4"
$env:OPENCV_LINK_PATHS="C:\vcpkg\installed\x64-windows\lib"
$env:OPENCV_INCLUDE_PATHS="C:\vcpkg\installed\x64-windows\include"
$env:LIBCLANG_PATH="C:\vcpkg\installed\x64-windows\bin"
$env:LLVM_CONFIG_PATH="C:\vcpkg\installed\x64-windows\tools\llvm\llvm-config.exe"
```

**注意**: PowerShellを閉じると設定がリセットされます。

---

### ステップ5: ビルドの実行

```powershell
# プロジェクトディレクトリに移動
cd C:\Users\benom\Develop\camera_app

# 以前のビルドをクリーンアップ
cargo clean

# ビルド実行
cargo build

# または、リリースビルド (最適化あり)
cargo build --release
```

**初回ビルド時間**: 約5-10分

---

## 🛠️ トラブルシューティング

### エラー1: `llvm-config` が見つからない

**症状**:
```
couldn't execute `llvm-config --prefix` (path=llvm-config) (error: program not found)
```

**解決策**:
`LLVM_CONFIG_PATH`環境変数を設定してください:
```powershell
[System.Environment]::SetEnvironmentVariable("LLVM_CONFIG_PATH", "C:\vcpkg\installed\x64-windows\tools\llvm\llvm-config.exe", "User")
```

---

### エラー2: `clang.dll` が見つからない

**症状**:
```
couldn't find any valid shared libraries matching: ['clang.dll', 'libclang.dll']
```

**解決策**:
`LIBCLANG_PATH`環境変数を設定してください:
```powershell
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\vcpkg\installed\x64-windows\bin", "User")
```

ファイルが存在するか確認:
```powershell
Test-Path "C:\vcpkg\installed\x64-windows\bin\libclang.dll"
```

`False`の場合、LLVMのインストールをやり直してください。

---

### エラー3: OpenCVライブラリが見つからない

**症状**:
```
Could not find OpenCV library
```

**解決策**:
OpenCVのパスが正しいか確認:
```powershell
# ライブラリファイルが存在するか確認
Test-Path "C:\vcpkg\installed\x64-windows\lib\opencv_world4.lib"

# 存在しない場合、OpenCVを再インストール
cd C:\vcpkg
.\vcpkg remove opencv4:x64-windows
.\vcpkg install opencv4[contrib,nonfree]:x64-windows
```

---

### エラー4: 環境変数が反映されない

**解決策**:
1. PowerShellを完全に再起動
2. VS Codeを再起動
3. PCを再起動 (最終手段)

環境変数が設定されているか確認:
```powershell
$env:OPENCV_LINK_LIBS
$env:LIBCLANG_PATH
```

空白の場合、環境変数が設定されていません。

---

## 📋 環境変数の確認コマンド

全ての環境変数が正しく設定されているか確認:

```powershell
Write-Host "=== 環境変数の確認 ===" -ForegroundColor Cyan
Write-Host "OPENCV_LINK_LIBS: $env:OPENCV_LINK_LIBS"
Write-Host "OPENCV_LINK_PATHS: $env:OPENCV_LINK_PATHS"
Write-Host "OPENCV_INCLUDE_PATHS: $env:OPENCV_INCLUDE_PATHS"
Write-Host "LIBCLANG_PATH: $env:LIBCLANG_PATH"
Write-Host "LLVM_CONFIG_PATH: $env:LLVM_CONFIG_PATH"

Write-Host "`n=== ファイルの存在確認 ===" -ForegroundColor Cyan
Write-Host "libclang.dll: $(Test-Path 'C:\vcpkg\installed\x64-windows\bin\libclang.dll')"
Write-Host "opencv_world4.lib: $(Test-Path 'C:\vcpkg\installed\x64-windows\lib\opencv_world4.lib')"
Write-Host "llvm-config.exe: $(Test-Path 'C:\vcpkg\installed\x64-windows\tools\llvm\llvm-config.exe')"
```

**期待される出力**:
```
=== 環境変数の確認 ===
OPENCV_LINK_LIBS: opencv_world4
OPENCV_LINK_PATHS: C:\vcpkg\installed\x64-windows\lib
OPENCV_INCLUDE_PATHS: C:\vcpkg\installed\x64-windows\include
LIBCLANG_PATH: C:\vcpkg\installed\x64-windows\bin
LLVM_CONFIG_PATH: C:\vcpkg\installed\x64-windows\tools\llvm\llvm-config.exe

=== ファイルの存在確認 ===
libclang.dll: True
opencv_world4.lib: True
llvm-config.exe: True
```

全て`True`になっていればOKです。

---

## 🚀 セットアップ完了後

セットアップが完了したら、以下のコマンドでアプリを実行できます:

```powershell
# 開発ビルド (デバッグ情報あり、遅い)
cargo run

# リリースビルド (最適化あり、速い、推奨)
cargo run --release
```

---

## 📖 参考リンク

- [vcpkg公式ドキュメント](https://vcpkg.io/)
- [opencv-rust GitHub](https://github.com/twistedfall/opencv-rust)
- [LLVM公式サイト](https://llvm.org/)
- [OpenCV公式サイト](https://opencv.org/)

---

## 💡 ヒント

### インストール時間の短縮

vcpkgは初回インストールに時間がかかりますが、以下で高速化できます:

1. **並列ビルド**: `vcpkg.exe`を実行する際、PowerShellを複数起動して他の作業を並行して行う
2. **バイナリキャッシュ**: 2回目以降は自動的にキャッシュが使われるため高速化
3. **SSDを使用**: HDD環境よりSSD環境の方が数倍速い

### ディスク容量の確保

vcpkgとOpenCV、LLVMのインストールには約**5-10GB**のディスク容量が必要です。
Cドライブの空き容量を確認してください:

```powershell
Get-PSDrive C | Select-Object Used,Free
```

---

**問題が解決しない場合は、Issueを作成してください: https://github.com/Moge800/camera_app/issues**
