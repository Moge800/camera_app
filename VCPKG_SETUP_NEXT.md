# vcpkgインストール完了後の手順

## ✅ インストール完了確認

LLVMとOpenCVのインストールが完了したら、以下のコマンドで確認してください:

```powershell
# インストール済みパッケージを確認
C:\vcpkg\vcpkg.exe list

# 以下が表示されればOK:
# llvm:x64-windows
# opencv4:x64-windows
```

## 🔧 環境変数の設定

```powershell
# 環境変数を永続的に設定
[System.Environment]::SetEnvironmentVariable("VCPKG_ROOT", "C:\vcpkg", "User")
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\vcpkg\installed\x64-windows\bin", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_LIBS", "opencv_world4", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_LINK_PATHS", "C:\vcpkg\installed\x64-windows\lib", "User")
[System.Environment]::SetEnvironmentVariable("OPENCV_INCLUDE_PATHS", "C:\vcpkg\installed\x64-windows\include", "User")

# PATHに追加
$userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*vcpkg\installed\x64-windows\bin*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$userPath;C:\vcpkg\installed\x64-windows\bin", "User")
}
```

## 🚀 ビルドと実行

```powershell
# VS Codeを再起動してから実行

# プロジェクトディレクトリに移動
cd C:\Users\benom\Develop\camera_app

# ビルド
cargo clean
cargo build

# 実行
cargo run --release
```

## 📝 注意事項

- **VS Codeの再起動が必須**: 環境変数を反映させるため
- **初回ビルドは時間がかかります**: 5-10分程度
- **エラーが出た場合**: このファイルの内容を確認して環境変数を再設定

## 🔍 トラブルシューティング

### エラー: `libclang.dll` が見つからない

```powershell
# DLLの存在確認
Test-Path "C:\vcpkg\installed\x64-windows\bin\libclang.dll"
# Trueが返ればOK

# Pathに追加されているか確認
$env:Path -split ';' | Where-Object { $_ -like '*vcpkg*' }
```

### エラー: OpenCVが見つからない

```powershell
# OpenCVライブラリの存在確認
Test-Path "C:\vcpkg\installed\x64-windows\lib\opencv_world4.lib"
# Trueが返ればOK

# 環境変数を再確認
$env:OPENCV_LINK_LIBS
$env:OPENCV_LINK_PATHS
$env:OPENCV_INCLUDE_PATHS
```

---

**インストールが完了したら、この手順を実行してください！**
