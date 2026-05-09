//! Windows context-menu integration via per-user registry keys.
//!
//! ## Where do the keys live?
//!
//! Per-user shell extensions live under `HKCU\Software\Classes` (the user
//! half of the merged `HKEY_CLASSES_ROOT` view). For each [`FileKind`] we
//! support — Audio, Video, Image, Document, Archive — we register a
//! single cascading verb under the **wildcard `*` class**, gated by an
//! `AppliesTo` Search Conditions filter:
//!
//! ```text
//! HKCU\Software\Classes\*\shell\Xolariq.<Kind>
//!   (default)    = ""                                  ; no direct command — this is a parent
//!   MUIVerb      = "Convert with Xolariq"
//!   Icon         = "<exe path>"                        ; bare, no quotes
//!   SubCommands  = ""                                  ; intentionally empty: see below
//!   AppliesTo    = "<Search Conditions query>"         ; per-kind, see FileKind::applies_to_query
//! HKCU\Software\Classes\*\shell\Xolariq.<Kind>\shell\01_To<Fmt>
//!   MUIVerb      = "To <Fmt>"
//! HKCU\Software\Classes\*\shell\Xolariq.<Kind>\shell\01_To<Fmt>\command
//!   (default)    = "<exe>" --target <fmt> --input "%1"
//! ...
//! ```
//!
//! ## Why the wildcard `*` class?
//!
//! Windows 11 (verified on 25H2 by the original PR author) silently
//! ignores classic shell verbs registered at
//! `HKCU\Software\Classes\.<ext>\shell\<verb>` even though the merged
//! `HKEY_CLASSES_ROOT` view shows them correctly. Verbs registered under
//! the wildcard class `HKCU\Software\Classes\*\shell\<verb>` *are* read
//! by Explorer in every Windows 10/11 build we tested. We compensate for
//! the wildcard's "applies to all files" semantics by attaching an
//! `AppliesTo` filter — see below.
//!
//! ## Why the cascade pattern?
//!
//! The submenu uses the documented "empty `SubCommands` + nested
//! `\shell\<verb>` subkeys" pattern. With `SubCommands` set to the empty
//! string, Explorer treats the parent verb as a placeholder and
//! enumerates cascade items from its own `shell` subkey, with no HKCR
//! ProgID indirection. We tried `ExtendedSubCommandsKey`-driven
//! cascades against an HKCU-only ProgID but Explorer renders them empty
//! in practice on per-user installs.
//!
//! Verb names are prefixed with `01_`, `02_`, ... to lock the visual
//! ordering: Explorer enumerates subkeys lexicographically.
//!
//! ## Why `AppliesTo` over a per-extension registration?
//!
//! `AppliesTo` is a Windows Search Conditions string evaluated by
//! Explorer against the right-clicked file's properties. We use:
//!
//! * `System.Kind:="music"` for audio files
//! * `System.Kind:="video"` for video files
//! * `System.Kind:="picture"` for image files
//! * `System.Kind:="document"` for document files
//! * `System.FileExtension:=".zip" OR …` for archives
//!
//! `System.Kind` is the cleanest filter — semantic, file-association
//! independent, and stable across Windows builds. Archives have no
//! `System.Kind` value (`compressed` `System.PerceivedType` is
//! unreliable on Windows 11 25H2), so we fall back to an explicit
//! extension list. The `:=` operator is required: an implicit colon
//! (`System.FileExtension:".zip"`) does not filter on Windows 11 25H2.
//!
//! ## Why not a `ContextMenuHandler`?
//!
//! A native COM `IExplorerCommand` handler is more powerful but requires
//! shipping a registered DLL and writing significantly more boilerplate.
//! The pure-registry approach used here is officially supported by
//! Explorer for cascading menus and fits Xolariq Free's "no DLLs, no
//! admin" constraint.
//!
//! ## Windows 11 caveat
//!
//! Windows 11's compact context menu only surfaces verbs registered as
//! `IExplorerCommand` COM objects. Classic registry verbs (including
//! ours) still work but appear under **"Show more options"** (or
//! `Shift + Right-click`). This is a Microsoft-side decision, not a
//! bug in this integration.

use std::path::{Path, PathBuf};
use std::process::Command;

use winreg::enums::*;
use winreg::RegKey;

use xolariq_core::{FileKind, Format, FormatList};

use super::{Result, ShellError, ShellIntegration};

/// Filename of the COM in-process server produced by the
/// `xolariq-shellext` crate. We look for it next to `xolariq.exe` and,
/// when found, drive it through `regsvr32` so the per-user
/// `IExplorerCommand` is also registered alongside the classic registry
/// verbs. The DLL is optional — the registry verbs alone are a fully
/// functional fallback (just hidden under "Show more options" on Win11).
const SHELLEXT_DLL_NAME: &str = "xolariq_shellext.dll";

const CLASSES_ROOT: &str = "Software\\Classes";
const WILDCARD_SHELL: &str = "Software\\Classes\\*\\shell";
const VERB_PREFIX: &str = "Xolariq";
const APP_DISPLAY: &str = "Convert with Xolariq";
const LEGACY_COMMAND_STORE: &str = "Software\\Classes\\Xolariq.CommandStore";
const LEGACY_PER_EXT_VERB: &str = "Xolariq";

pub struct WindowsShellIntegration;

impl WindowsShellIntegration {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsShellIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellIntegration for WindowsShellIntegration {
    fn install(&self, exe_path: &Path) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // Migrate cleanly from older registry layouts that targeted
        // `HKCU\Software\Classes\.<ext>\shell\Xolariq` (Windows 11 25H2
        // ignored those) and `HKCU\Software\Classes\Xolariq.CommandStore`
        // (HKCU ProgID-driven cascade). Failures here are not fatal.
        cleanup_legacy_keys(&hkcu);

        // Prefer the COM `IExplorerCommand` handler when the
        // shell-extension DLL is colocated with the running executable.
        // The COM handler renders the full flyout with icon, so the
        // classic per-kind registry verbs are not needed and would cause
        // a duplicate "Convert with Xolariq" entry in the context menu.
        if let Some(dll) = locate_shellext_dll(exe_path) {
            // Remove any leftover per-kind registry verbs from a previous
            // install so they don't duplicate the COM entry.
            for &kind in FileKind::ALL {
                let path = format!("{WILDCARD_SHELL}\\{verb}", verb = verb_name(kind));
                let _ = hkcu.delete_subkey_all(&path);
            }
            let _ = run_regsvr32(&dll, RegsvrAction::Register);
            return Ok(());
        }

        // Fallback: no COM DLL available — install per-kind registry
        // verbs (shown under "Show more options" on Win11).
        for &kind in FileKind::ALL {
            ensure_kind_menu(&hkcu, kind, exe_path)?;
        }
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // Try to unregister the COM handler before tearing down the
        // classic verbs, mirroring `install`.
        if let Some(dll) = locate_shellext_dll(&current_exe_or_self()) {
            let _ = run_regsvr32(&dll, RegsvrAction::Unregister);
        }

        for &kind in FileKind::ALL {
            let path = format!("{WILDCARD_SHELL}\\{verb}", verb = verb_name(kind));
            let _ = hkcu.delete_subkey_all(&path);
        }

        cleanup_legacy_keys(&hkcu);
        Ok(())
    }

    fn is_installed(&self) -> Result<bool> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        // Check for the COM handler's root verb first (the preferred path
        // when the DLL is present), then fall back to probing for the
        // classic per-kind registry verbs.
        let com_probe = format!("{WILDCARD_SHELL}\\XolariqRoot");
        if hkcu.open_subkey(&com_probe).is_ok() {
            return Ok(true);
        }
        let legacy_probe = format!(
            "{WILDCARD_SHELL}\\{verb}",
            verb = verb_name(FileKind::Audio)
        );
        Ok(hkcu.open_subkey(&legacy_probe).is_ok())
    }
}

/// Look for `xolariq_shellext.dll` next to a known executable path. Used
/// during install / uninstall to optionally drive the COM registration
/// without requiring the caller to know whether the bundle ships the
/// DLL.
fn locate_shellext_dll(exe_path: &Path) -> Option<PathBuf> {
    let parent = exe_path.parent()?;
    let candidate = parent.join(SHELLEXT_DLL_NAME);
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

/// Helper for the `uninstall` path which doesn't have a CLI-passed
/// `exe_path` available. Falls back to `current_exe()` and then to a
/// best-guess `xolariq.exe` lookup, both of which should resolve to the
/// installed location at uninstall time.
fn current_exe_or_self() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("xolariq.exe"))
}

#[derive(Clone, Copy)]
enum RegsvrAction {
    Register,
    Unregister,
}

/// Run the system `regsvr32` against `dll` to (un)register the COM
/// handler. We pass `/s` for silent operation and rely on the DLL's own
/// `DllRegisterServer` / `DllUnregisterServer` writing to `HKCU` (no
/// admin prompt). Failures are surfaced as [`ShellError::Io`] but the
/// caller is expected to swallow them — registration is best-effort.
fn run_regsvr32(dll: &Path, action: RegsvrAction) -> Result<()> {
    let mut cmd = Command::new("regsvr32.exe");
    cmd.arg("/s");
    if matches!(action, RegsvrAction::Unregister) {
        cmd.arg("/u");
    }
    cmd.arg(dll);
    let status = cmd.status()?;
    if !status.success() {
        return Err(ShellError::Registry(format!(
            "regsvr32 exited with {:?} for {}",
            status.code(),
            dll.display()
        )));
    }
    Ok(())
}

fn cleanup_legacy_keys(hkcu: &RegKey) {
    for &fmt in Format::ALL {
        let old = format!(
            "{CLASSES_ROOT}\\.{ext}\\shell\\{LEGACY_PER_EXT_VERB}",
            ext = fmt.extension()
        );
        let _ = hkcu.delete_subkey_all(&old);
    }
    let _ = hkcu.delete_subkey_all(LEGACY_COMMAND_STORE);
}

fn verb_name(kind: FileKind) -> String {
    format!("{VERB_PREFIX}.{}", kind.verb_suffix())
}

fn ensure_kind_menu(hkcu: &RegKey, kind: FileKind, exe_path: &Path) -> Result<()> {
    let targets: Vec<Format> = FormatList::for_kind(kind);
    if targets.is_empty() {
        return Ok(());
    }

    let parent_path = format!("{WILDCARD_SHELL}\\{verb}", verb = verb_name(kind));

    // Drop any previous tree under this verb so re-installs don't leave
    // dead subkeys behind (e.g. when the per-kind format list shrinks).
    let _ = hkcu.delete_subkey_all(&parent_path);

    let (parent, _) = hkcu
        .create_subkey(&parent_path)
        .map_err(|e| ShellError::Registry(format!("create {parent_path}: {e}")))?;

    parent
        .set_value("MUIVerb", &APP_DISPLAY.to_string())
        .map_err(|e| ShellError::Registry(format!("set MUIVerb: {e}")))?;

    // The Icon value must be a bare path (optionally followed by `,index`).
    // Wrapping it in quotes makes Explorer treat the literal `"..."` string
    // as a missing file and render no glyph at all.
    parent
        .set_value("Icon", &exe_path.to_string_lossy().to_string())
        .map_err(|e| ShellError::Registry(format!("set Icon: {e}")))?;

    // An *empty* SubCommands value is the documented opt-in for the
    // nested-shell cascade pattern. With it set, Explorer enumerates
    // <parent>\shell\<verb> subkeys instead of consulting any command
    // store. Omitting SubCommands entirely would tell Explorer to treat
    // the verb as a single non-cascading command and look for a `command`
    // subkey directly.
    parent
        .set_value("SubCommands", &"".to_string())
        .map_err(|e| ShellError::Registry(format!("set SubCommands: {e}")))?;

    // Filter the verb to files of this kind. See FileKind::applies_to_query
    // for the exact Windows Search Conditions strings.
    parent
        .set_value("AppliesTo", &kind.applies_to_query())
        .map_err(|e| ShellError::Registry(format!("set AppliesTo: {e}")))?;

    let exe = exe_path.to_string_lossy().to_string();

    for (idx, target) in targets.iter().enumerate() {
        // Numeric prefix locks the visual ordering — Explorer enumerates
        // subkeys alphabetically.
        let verb_subkey = format!("{:02}_To{}", idx + 1, upper_first(target.extension()));
        let verb_path = format!("{parent_path}\\shell\\{verb_subkey}");

        let (verb, _) = hkcu
            .create_subkey(&verb_path)
            .map_err(|e| ShellError::Registry(format!("create {verb_path}: {e}")))?;

        verb.set_value("MUIVerb", &format!("To {}", target.label()))
            .map_err(|e| ShellError::Registry(format!("set MUIVerb: {e}")))?;

        let (cmd_key, _) = verb
            .create_subkey("command")
            .map_err(|e| ShellError::Registry(format!("create command: {e}")))?;

        // %1 is the path of the file the user right-clicked. Both the exe
        // path and the input path are wrapped in literal quotes so paths
        // containing spaces survive Explorer's argv split.
        let cmd = format!(
            "\"{exe}\" --target {ext} --input \"%1\"",
            ext = target.extension()
        );
        cmd_key
            .set_value("", &cmd)
            .map_err(|e| ShellError::Registry(format!("set command: {e}")))?;
    }

    // "Settings" entry — launches xolariq.exe with no arguments, which
    // opens the Settings window. Placed at the end of the cascade.
    let settings_idx = targets.len() + 1;
    let settings_subkey = format!("{:02}_Settings", settings_idx);
    let settings_path = format!("{parent_path}\\shell\\{settings_subkey}");
    let (settings_verb, _) = hkcu
        .create_subkey(&settings_path)
        .map_err(|e| ShellError::Registry(format!("create {settings_path}: {e}")))?;
    settings_verb
        .set_value("MUIVerb", &"Settings".to_string())
        .map_err(|e| ShellError::Registry(format!("set MUIVerb: {e}")))?;
    // Use settings.ico — check next to the exe first, then in the
    // `external/` resources subdirectory where Tauri MSI places it.
    if let Some(ico) = exe_path.parent().and_then(|d| {
        let direct = d.join("settings.ico");
        if direct.is_file() {
            return Some(direct);
        }
        let resource = d.join("external").join("settings.ico");
        if resource.is_file() {
            return Some(resource);
        }
        None
    }) {
        settings_verb
            .set_value("Icon", &ico.to_string_lossy().to_string())
            .map_err(|e| ShellError::Registry(format!("set Icon: {e}")))?;
    }
    // Draw a separator line above this entry (ECF_SEPARATORBEFORE = 0x40).
    settings_verb
        .set_value("CommandFlags", &0x40u32)
        .map_err(|e| ShellError::Registry(format!("set CommandFlags: {e}")))?;

    let (settings_cmd, _) = settings_verb
        .create_subkey("command")
        .map_err(|e| ShellError::Registry(format!("create settings command: {e}")))?;
    // No arguments — the app opens the Settings window by default.
    settings_cmd
        .set_value("", &format!("\"{exe}\""))
        .map_err(|e| ShellError::Registry(format!("set settings command: {e}")))?;

    Ok(())
}

fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}
