# Countdown timer

[English](README.md) | [繁體中文](README.zh-TW.md)

一個常駐在最上層的小型倒數計時器，用來提醒讓眼睛休息。
計時長度可自行設定，預設 25 分鐘。

這個程式需要一直保持在畫面上，讓使用者可以隨時留意倒數狀況；除了用
滑鼠點擊重設時間之外，幾乎不需要操作。為了避免被誤切換或誤點，它刻意
不出現在 macOS 的 Dock 與 `Cmd`+`Tab`，也不出現在 Windows 的工作列與
`Alt`+`Tab`。

時間到時同樣採取軟性提醒：視窗內的數字轉成紅色並開始閃爍，除此之外
沒有通知、沒有音效，也不會跳出任何視窗搶走焦點。歸零後還會繼續往
負數累計，顯示已經超時多久，讓使用者能先把手邊的事收尾再休息。

## 安裝

免安裝，沒有安裝程式。使用者將下載的檔案解壓縮後，放到平常開啟程式
的位置，再手動開啟即可。Linux 版是單一檔案，連解壓縮都不需要。

到 [Releases](../../releases) 下載最新版本。

| 平台 | 檔案 |
| --- | --- |
| macOS（Apple 晶片） | `…-macos-arm64.zip` |
| Windows（x64） | `…-windows-x64.7z` |
| Linux（x64） | `…-linux-x64.AppImage` |

第一次執行前各需要一個步驟：

- **macOS** —— 有 ad-hoc 簽章但未經 Apple 公證，要先清掉隔離屬性才能
  開啟：`xattr -dr com.apple.quarantine <countdown.app 的路徑>`。把
  app 拖進終端機視窗即可自動填入路徑。
- **Windows** —— 執行檔沒有簽章，SmartScreen 會擋下第一次執行，點
  **其他資訊** → **仍要執行**。
- **Linux** —— 用 `chmod +x` 讓 AppImage 可執行。

## 操作

| | |
| --- | --- |
| `p` | 暫停／繼續 |
| `f` | 進入時間輸入模式（會暫停倒數） |
| `Enter` | 確認新時間並重新開始 |
| `Esc` | 離開輸入模式，不做任何更動 |
| 點擊時間 | 重設為上次設定的時間 |
| `Cmd`+`Q` | 結束程式（macOS） |
| `Alt`+`F4` | 結束程式（Windows） |

輸入模式下直接輸入數字，冒號會自動補上。範圍是 `00:00`–`59:59`。

## 行為說明

- 到 `00:00` 之後畫面會閃爍並繼續往負數倒數，到 `-59:59` 才停止。
- 設定的時間會被記住，重開程式後仍然沿用；第一次執行是 25:00。
- 沒有關閉按鈕也沒有選單列，要結束請用鍵盤。

## 從原始碼建置

需要 Rust、[Trunk](https://trunkrs.dev) 和 Tauri CLI：

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk tauri-cli

cargo tauri dev      # 執行
cargo tauri build    # 建置發行版
```

## 授權

MIT，詳見 [LICENSE](LICENSE)。
