# Running the test suite on Windows 11 (KVM/virt-manager VM)

The whole suite runs on Windows: frontend tests, typecheck, Rust unit tests,
Rust integration tests (real git repos) and the WebDriver E2E that drives the
real app window. Run everything from **Git Bash** (installed with Git for
Windows) — `verify.sh` and `e2e/run.sh` are bash scripts.

## 1. Install the toolchain (inside the Windows VM)

PowerShell (admin) — winget where possible:

```powershell
winget install Git.Git
winget install OpenJS.NodeJS.LTS          # Node 22
winget install Rustlang.Rustup            # rustup; install the MSVC toolchain when prompted
winget install pnpm.pnpm                  # pnpm 9+
winget install Microsoft.VisualStudio.2022.BuildTools
```

VS Build Tools needs the **"Desktop development with C++"** workload (linker
for Rust MSVC). Easiest way: after winget finishes run

```powershell
rustup default stable
```

and if the linker is missing, open **Visual Studio Installer → Build Tools →
Modify → Desktop development with C++** and install it.

Close and reopen terminals so PATH updates. Check:

```powershell
git --version; node --version; pnpm --version; cargo --version
```

## 2. Get the project into the VM

Pick one:

- **Git remote (recommended):** commit the work on the host, push to GitHub (or
  any remote), then `git clone` inside the VM. Note: uncommitted work is not
  pushed — commit first.
- **virtiofs shared folder (virt-manager):** VM details → *Add Hardware →
  Filesystem*, driver `virtiofs`, source = the host project folder. In the
  guest install [WinFsp](https://winfsp.dev/) and the virtiofs driver from the
  [virtio-win ISO](https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/) —
  the share then shows up as a drive letter.
- **One-shot copy:** on the host, archive the tree without build artifacts and
  move it over (SPICE folder sharing, SMB, or any file transfer):

  ```bash
  tar czf mikrogit.tgz \
    --exclude node_modules --exclude target --exclude build \
    --exclude .svelte-kit --exclude e2e/fixture-repo \
    -C .. mikrogit
  ```

Then inside the VM: `pnpm install`.

## 3. Fast suite (no UI)

Git Bash, project root:

```bash
./verify.sh --fast
```

Runs vitest, svelte-check, `cargo test --lib`, and the Rust integration tests
against real git repos. Fixtures set `core.autocrlf=false` per repo, so they
pass regardless of your global autocrlf setting.

## 4. Full suite with UI E2E

On Windows, tauri-driver drives **WebView2** through `msedgedriver`
(instead of Linux's WebKitWebDriver).

1. Build the app once (also produces the release binary the E2E launches):

   ```bash
   pnpm tauri build
   ```

2. Find your WebView2 Runtime version:

   ```powershell
   (Get-ChildItem "$env:ProgramFiles(x86)\Microsoft\EdgeWebView\Application").Name
   ```

3. Download the **matching** driver and put `msedgedriver.exe` on PATH:

   ```
   https://msedgedriver.azureedge.net/<version>/edgedriver_win64.zip
   ```

   e.g. runtime `138.0.3351.65` → `.../138.0.3351.65/edgedriver_win64.zip`.
   Unzip and copy `msedgedriver.exe` into `%USERPROFILE%\.cargo\bin`
   (already on PATH).

4. `cargo install tauri-driver --locked` (skip if done before).

5. Run everything:

   ```bash
   ./verify.sh
   ```

The app window appears briefly while the E2E drives stage → commit → explorer
→ terminal. Auto-rebuild of a stale binary works the same as on Linux.

## Notes / troubleshooting

- **Port 4444 busy:** `E2E_PORT=4445 ./verify.sh`.
- **Long-path errors** from node_modules: enable long paths once
  (admin PowerShell): `New-ItemProperty -Path HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem -Name LongPathsEnabled -Value 1 -PropertyType DWord -Force`
- **Slow builds:** exclude the project folder and `%LOCALAPPDATA%\cargo` /
  `target` from Defender real-time scanning while developing.
- **`msedgedriver` version mismatch:** WebView2 auto-updates; when the version
  changes, download the matching driver again. Symptom: E2E hangs at session
  start with a version error in the tauri-driver output.
- **msedgedriver location:** any directory on PATH works; tauri-driver spawns
  it automatically.
