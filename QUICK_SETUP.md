# 簡易セットアップガイド

OpenCVの完全なビルドには時間がかかるため、以下の簡易的な方法をお試しください。

## オプション1: プリビルド版OpenCVを使用

### 1. LLVMのインストール

公式サイトからLLVMインストーラをダウンロード:
https://github.com/llvm/llvm-project/releases

**推奨**: LLVM-18.1.8-win64.exe

インストール後:
```powershell
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")
```

### 2. OpenCVのダウンロード

公式サイトからOpenCVをダウンロード:
https://opencv.org/releases/

**推奨**: opencv-4.10.0-windows.exe

解凍後 (例: C:\opencv):
```powershell
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_LIBS", "opencv_world4100", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_PATHS", "C:\opencv\build\x64\vc16\lib", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_INCLUDE_PATHS", "C:\opencv\build\include", "User")

# DLLのパスも追加 (重要!)
$path = [System.Environment]::GetEnvironmentVariable("Path", "User")
[System.Environment]::SetEnvironmentVariable("Path", "$path;C:\opencv\build\x64\vc16\bin", "User")
```

### 3. VS Codeを再起動

環境変数を反映させるため、VS Codeを完全に閉じて再起動してください。

### 4. ビルド

```powershell
cd C:\Users\benom\Develop\camera_app
cargo clean
cargo build
```

---

## オプション2: vcpkgを使用 (完全版、時間がかかる)

### 前提条件
- Visual Studio 2022 (C++ワークロード) がインストールされている必要があります
- 5-10GBのディスク空き容量
- 2-3時間の待ち時間 (初回のみ)

### 手順

```powershell
# 1. vcpkgを設定
cd C:\vcpkg
.\vcpkg integrate install

# 2. LLVMをインストール (30-60分)
.\vcpkg install llvm:x64-windows

# 3. OpenCVをインストール (20-40分)
.\vcpkg install opencv4:x64-windows

# 4. 環境変数を設定
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_LIBS", "opencv_world4", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_PATHS", "C:\vcpkg\installed\x64-windows\lib", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_INCLUDE_PATHS", "C:\vcpkg\installed\x64-windows\include", "User")
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\vcpkg\installed\x64-windows\bin", "User")

# 5. VS Codeを再起動

# 6. ビルド
cd C:\Users\benom\Develop\camera_app
cargo clean
cargo build
```

---

## 推奨: オプション1を選択

オプション1の方が、セットアップが早く完了します (約10-15分)。
オプション2は、より統合された環境を構築できますが、初回セットアップに2-3時間かかります。

どちらを選びますか?
