<p align="center">
  <img src="assets/icon.png" width="80" />
</p>

<h1 align="center">Skills Manager</h1>

<p align="center">
  一個應用，統一管理所有 AI 編碼工具的 Skills。
</p>

<p align="center">
  <strong><a href="https://github.com/StonesA3A2/Skill-Manager">github.com/StonesA3A2/Skill-Manager</a></strong>
</p>

<p align="center">
  <a href="./README.md">English</a> &nbsp;·&nbsp;
  <a href="./README.de.md">Deutsch</a> &nbsp;·&nbsp;
  <a href="./README.zh-CN.md">简体中文</a> &nbsp;·&nbsp;
  <b>繁體中文</b>
</p>

<p align="center">
  <img src="assets/demo/library.png" width="800" alt="Skills Manager 技能庫" />
</p>

<p align="center"><strong>安裝 Skills</strong></p>
<p align="center"><img src="assets/demo/install-skills.png" width="800" alt="安裝 Skills" /></p>

<p align="center"><strong>全域工作區</strong></p>
<p align="center"><img src="assets/demo/global-workspace.png" width="800" alt="全域工作區" /></p>

<p align="center"><strong>Agent 工作區</strong></p>
<p align="center"><img src="assets/demo/agent-workspace.png" width="800" alt="Agent 工作區" /></p>

<p align="center"><strong>專案工作區</strong></p>
<p align="center"><img src="assets/demo/project-workspace.png" width="800" alt="專案工作區" /></p>

<p align="center"><strong>備份與多裝置同步</strong></p>
<p align="center"><img src="assets/demo/backup.png" width="800" alt="備份與多裝置同步" /></p>

<p align="center"><strong>設定</strong></p>
<p align="center"><img src="assets/demo/settings.png" width="800" alt="設定" /></p>

## 功能

- **統一技能庫** — 從 Git 倉庫、本地目錄、`.zip` / `.skill` 檔案或 [skills.sh](https://skills.sh) 市集安裝技能，統一存放在 `~/.skills-manager`。
- **Preset（預設）** — 將技能分組為命名 Preset。在任意工作區點擊 Preset 標籤，即可一鍵為目前 Agent 範圍啟用或停用其全部技能，啟用的 Preset 顯示 ✓，部分安裝顯示數量。
- **全域工作區** — 每個 Agent 都有自己的頁面，列出其全域目錄裡的所有 Skills（包括不是透過 Skills Manager 安裝的），始終反映 Agent 實際看到的內容。可依 Agent 新增或移除 Skills，也可透過「所有 Agents」總覽跨所有已安裝 Agent 統一管理。
- **專案工作區** — 檢視並管理任意專案的本地 Skills 目錄，支援與中央庫雙向同步。支援巢狀 Skill 目錄和匯出時依 Agent 分配。
- **關聯工作區** — 將任意目錄指定為 Skills 根目錄，適合管理不在預設 Agent 路徑下的 Skills。作為獨立工作區管理，不參與全域 Preset 同步。
- **多工具同步** — 一鍵將技能同步到任意支援的工具，支援軟連結和複製兩種模式。每張 Skill 卡片會為每個已啟用 Agent 顯示一個圖示角標，點擊角標即可直接在卡片上為該 Agent 安裝或移除這個 Skill，角標會即時反映同步狀態。
- **「新增 Skills」彈層** — 任意工作區點擊 **+ 新增 Skills** 即可開啟統一的挑選彈層：搜尋中央庫，用始終可見的 Agent 標籤切換目標（含一鍵全選/清空），一次提交批次新增多個 Skills。
- **批次操作** — 多選技能後批次啟用/停用、匯出或刪除。專案工作區中的專案 Skills 也支援批次啟用/停用。
- **技能標籤** — 為技能加上標籤，用於歸類同類技能，並依來源或標籤篩選；新增的 **未標籤** 篩選項可快速定位漏打標籤的 Skills。
- **更新檢查** — 為 Git 類技能檢查遠端更新；本地技能支援重新匯入。
- **文件預覽** — 直接在應用內檢視 `SKILL.md` / `README.md`。
- **自訂工具** — 新增自訂 Agent/工具並指定 Skills 目錄，也可覆蓋內建工具的預設路徑。
- **備份與多裝置同步** — 一次 GitHub 登入（或任意 Git 遠端）接入私有備份倉庫，之後自動備份、多台裝置自動保持一致。合併以技能為單位——一台改名、另一台改內容會自動組合；真衝突不阻擋不覆蓋，本機版本保留待你三選一處理。快照版本隨時可還原。
- **活動紀錄 & 匯出紀錄** — 應用會記錄本地的安裝/移除/更新/同步操作。在 **設定 → 匯出紀錄** 可把最近紀錄和活動記錄打包成壓縮檔，方便提交 Issue 時附上。
- **彈性的應用設定** — 在一個頁面裡設定倉庫路徑、同步模式、主題、字型大小、語言、系統匣行為、代理伺服器、Git 遠端、更新檢查，以及 Agent 在全應用中的顯示順序。
- **應用內更新** — 有新版本時應用會主動提醒，並在 macOS 和 Windows 上直接完成安裝。不會自行下載或安裝：檢查只負責告知，安裝和重新啟動各需一次點擊。

## 核心概念

- **Preset 是可重複使用的 Skills 分組** — Preset 是一組命名的 Skills 集合。在任意工作區啟用 Preset，即可將其所有 Skills 新增到選定 Agent；停用則反向移除。套用 Preset 是一次性複製，不是即時同步。
- **全域工作區管理每個 Agent 的全域 Skills** — 每個已安裝 Agent 都有自己的全域 Skills 目錄（如 Claude Code 對應 `~/.claude/skills/`）。每個 Agent 頁面會列出該目錄裡的所有內容（包括不是透過 Skills Manager 安裝的 Skills），可以新增、移除或納入管理；「所有 Agents」總覽則跨 Agent 統一管理。
- **專案工作區是專案專屬 Skills 集合** — 專案工作區管理某個專案裡的本地 Skills（如 `<project>/.claude/skills/`），只對該專案生效。
- **標籤用於歸類和篩選** — 給同類 Skills 打上相同標籤後，可以依標籤快速篩選出需要的一組 Skills。
- **批次操作隨處可用** — 在任意工作區多選 Skills，進行批次操作。

## 快速上手

1. 從本地目錄、Git 倉庫、壓縮檔或市集安裝 Skills。
2. 從側邊欄進入 **全域工作區**，選擇一個 Agent（如 Claude Code）。
3. 點擊 **Preset** 標籤為該 Agent 一鍵啟用對應 Skills，或點 **+ 新增 Skills** 從技能庫挑選並即時切換目標 Agent。啟用的 Preset 顯示 ✓，部分安裝顯示計數角標。
4. 如需管理專案本地 Skills，開啟 **專案工作區**，同樣使用 Preset 標籤，或透過 **+ 新增 Skills** 彈層用多 Agent 目標選擇器挑選。
5. 在 **設定** 中設定 Agent 路徑、自訂工具、主題、語言、代理伺服器和 Git 偏好設定。
6. 如果需要歷史版本或多機同步，從側邊欄開啟 **備份** 頁，點擊 **使用 GitHub 登入**——之後備份和跨裝置同步都會自動進行。

## 備份與多裝置同步

側邊欄的 **備份** 頁把技能庫託管在一個 Git 倉庫裡：單台裝置是帶版本歷史、可還原快照的備份；多台裝置連接同一倉庫時會自動保持一致。遠端始終是純 Git 倉庫——隨時可以 `git clone` 走，沒有鎖定。

### 連接

- **使用 GitHub 登入**（推薦）：輸入 8 位碼完成授權，應用會自動建立私有倉庫 `skills-manager-backup`。權杖只存在系統金鑰圈裡，絕不落入檔案或倉庫設定。
- **進階方式**：在 **設定 → Git 同步設定** 貼上任意 Git 位址（HTTPS + PAT、SSH、自建服務均可）。
- 新機器上技能庫為空時，首次啟動會詢問：**全新開始，還是從備份還原？**

### 同步如何運作

- **全自動**：本地變更停止編輯約兩分鐘後自動提交並上傳；其他裝置推送的更新會自動合併進來並推送回去。隨時可點 **立即備份**，備份歷史會顯示每一筆來自哪台裝置。
- **依技能合併**：同步以技能為單位而非文字行——一台裝置改名、另一台改內容，會自動正確組合。
- **衝突不阻擋、不覆蓋**：同一技能在兩台裝置被同時修改時，其餘技能照常同步，該技能保留本機版本並進入 **需要處理** 清單（技能卡上也有徽章）。三選一：**保留本機 / 使用遠端 / 兩者都保留**——套用任一選擇前都會先建安全快照，每個決定都可復原。
- **快照與還原**：手動備份會建立快照版本，在備份頁歷史中可還原任意一個；還原前會先把目前狀態存為新快照。

### 備份包含什麼

技能檔案、標籤、Preset 及每個 Agent 的技能開關會被備份。機密資訊（API 金鑰、權杖、代理伺服器設定）和本機接線永不上傳。超過 100 MB 的技能自動留在本機、不進備份（備份頁會標註）。SQLite 資料庫不進 Git——其中的中繼資料可從技能檔案重建。

### 中斷連接

備份頁提供三檔：**中斷本機**（其他裝置與遠端資料不受影響）、**撤銷 GitHub 授權**、以及 **刪除遠端備份**（經 GitHub 原生的輸入倉庫名稱確認流程）。

## 支援的工具

開箱支援 52 個 Agent，包括：

Claude Code · Codex · Cursor · GitHub Copilot · Gemini CLI · OpenCode · OpenClaw · Hermes Agent · OpenHands · Cline · Goose · Windsurf · Continue · Grok · Antigravity · Qwen Code · Crush · Kilo Code · Roo Code · Amp · Kiro CLI · Droid · TRAE IDE · Warp · Qoder · CodeBuddy

**設定**頁會列出全部，並優先顯示在你機器上偵測到的那些。你也可以在那裡新增自訂工具，以相同方式管理其 Skills。

## 應用內說明

設定頁中的 **說明** 按鈕會展示與上面一致的快速流程：推薦工作流、Preset、安裝 Skills、技能庫（含「未標籤」篩選與卡片刪除按鈕）、全域工作區與 **+ 新增 Skills** 彈層、專案工作區的多 Agent 目標選擇器、備份與多裝置同步，以及環境設定（含「匯出紀錄」用於 Issue 回報），方便使用者不離開應用也能快速理解使用方式。

## 技術棧

| 層 | 技術 |
|----|------|
| 前端 | React 19、TypeScript、Vite、Tailwind CSS |
| 桌面 | Tauri 2 |
| 後端 | Rust |
| 儲存 | SQLite（`rusqlite`） |
| 國際化 | react-i18next |

## 快速開始

### 前置需求

- Node.js 18+
- Rust 工具鏈
- 目前系統的 [Tauri 相依套件](https://v2.tauri.app/start/prerequisites/)

### 開發

```bash
npm install
npm run tauri:dev
```

### CLI

倉庫現在包含一個面向 agent 的 CLI，而且它是建立在與桌面應用共用的 Rust shared core 之上。

```bash
# 查看目前倉庫路徑和統計資訊
npm run cli -- repo status

# 列出技能 / 查看單一技能
npm run cli -- skills list
npm run cli -- skills show db

# 把中央庫 skill 部署給具體 Agent
npm run cli -- skills deploy db --agent claude_code --agent codex
npm run cli -- skills status db

# 把已安裝的技能改指向 git 來源，技能 id、標籤、Preset 歸屬和既有部署都保留
npm run cli -- skills set-source db --git-url https://github.com/you/skills/tree/main/db --dry-run
npm run cli -- skills set-source db --git-url you/skills --subpath db --force

# 管理和部署 Preset（CRUD/成員調整只整理資料，deploy 才修改 Agent 檔案）
npm run cli -- presets create "Web Dev" --description "前端開發"
npm run cli -- presets add-skill "Web Dev" db
npm run cli -- presets deploy "Web Dev" --agent codex
npm run cli -- presets status "Web Dev"

# 匯出單一技能到其他 agent 工作目錄
npm run cli -- skills export db --dest ~/.claude/skills/db

# 查看或同步 git 管理的 skills 倉庫
npm run cli -- git status
npm run cli -- git pull
npm run cli -- git commit -m "chore: update skills"
```

可用指令分組：
- `repo`：查看或修改目前 base directory
- `agents`（相容別名 `tools`）：列出 Agent，並全域啟用或停用 Agent
- `skills`：管理中央庫、標籤，以及 skill 在各 Agent 中的實際部署
- `presets`：建立、修改、刪除、整理、部署或撤下 Preset
- `git`：操作 git 管理的 `skills/` 倉庫（`clone`、`pull`、`push`、`commit`、`versions`、`restore`）

額外參數：
- `--skills-root <path>`：直接針對某個已 clone / 已匯出的 skills repo 操作，而不是本機 app 預設目錄。manager 的狀態（DB、scenarios、cache、logs）會落在 `~/.skills-manager/external/<name>-<hash>/`，依 skills root 的正規化路徑分目錄隔離，外部倉庫本身保持乾淨。
- `--json`：給指令碼 / agent 使用的機器可讀輸出

```bash
npm run -s cli -- --skills-root /path/to/my-skills --json skills list
```

#### 把 CLI 執行檔安裝到 PATH

如果 agent / 指令碼直接呼叫 `skills-manager-cli`（而不是 `npm run`），需要先把執行檔放到 PATH 上：

```bash
npm run cli:install
# 等同於：
# cargo install --path src-tauri --bin skills-manager-cli --locked --force
```

執行檔會裝到 `~/.cargo/bin/skills-manager-cli`。程式碼更新後再跑一次即可刷新。

正式 Release 也會提供 macOS arm64/x64、Windows x64、Linux x64 的獨立 CLI 檔案。下載對應的 `skills-manager-cli-*`，在 macOS/Linux 加上可執行權限後放入 PATH 即可。

#### 與桌面應用並行使用

CLI 和桌面應用共享同一個 SQLite 資料庫及倉庫鎖。CLI 修改中繼資料或 Agent 部署後，桌面應用通常會透過檔案監控自動刷新；如果應用當時處於休眠狀態，手動刷新一次即可。

### 建置

```bash
npm run tauri:build
npm run cli:build
```

## 常見問題

### macOS 首次啟動被 Gatekeeper 攔截（v1.28.5 及之前）

**v1.29.0** 起的版本使用 Apple Developer ID 憑證簽署並經過 Apple 公證，可以直接開啟——不會有警告，也不需要在終端機裡輸入指令。如果你還在用舊版本，升級即可解決。

**v1.28.5 及之前的版本**發布於公證之前，會被 macOS 攔截：

<p align="center">
  <img src="assets/CleanShot_20260530_093302@2x.png" width="320" alt="macOS Gatekeeper 提示：無法驗證 skills-manager.app 是否包含惡意軟體" />
</p>

- **"無法驗證此 App 是否包含惡意軟體"** 或 **"無法打開，因為無法驗證開發者"**（v1.20.0 – v1.28.5）—— 在 macOS 15（Sequoia）上，上圖的彈窗只有 **移到垃圾桶** / **完成** 兩個按鈕：點 **完成**，再打開 **系統設定 → 隱私權與安全性**，點 **仍要打開**（第一次被攔截後會出現）。舊版 macOS 也可以在 Finder 裡按右鍵點擊應用、選擇 **打開**，再在彈窗裡確認。
- **"應用程式已損毀，無法打開"**（v1.19.0 及之前版本）—— 在終端機執行下面這條指令後重新打開應用即可：

  ```bash
  xattr -cr /Applications/skills-manager.app
  ```

  如果 `.app` 不在 `/Applications`，請替換為實際路徑。

升級到公證版本時，應用的程式碼簽章發生了變化，macOS 可能會再問一次是否允許讀取 `skills-manager-git-backup` 金鑰圈項目。點 **永遠允許** 即可——從 v1.29.0 起簽章身分保持穩定，之後的更新應該不會再問。

## License

MIT
