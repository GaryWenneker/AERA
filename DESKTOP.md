# Desktop 0.1 — information architecture

Visual reference: the user supplied desktop examples with a quiet wallpaper, selectable application icons, dock, file browser with sidebar and organized settings. AERA adopts their interaction hierarchy, not their logos or exact appearance.

Desktop: persistent bottom dock during session, application grid, searchable launcher, independent movable internal windows, active application markers. Files: sidebar Home / Computer / parent, navigation toolbar, path, directory table with name/type/size, selected item and status. Double-click directories, no execution of files. Disks: device or mount inventory, filesystem, total/available capacity when reported, refresh, error and unsupported states. Settings: appearance, scale, storage shortcut, system information. Monitor and Terminal are subsequent first-party applications, not decorative placeholders. Installer is deferred until recovery tests.

## Safety and acceptance
- [ ] AERA-028 Launcher offers real Files, Disks, Settings, About actions with visible open state.
- [ ] AERA-029 Files navigates real directories, parent and home, reports errors and size/type; first increment read-only, bounded listing. Later create/rename/copy/move/trash with confirmation, collision handling and recovery.
- [ ] AERA-030 Disks displays real mounted filesystem capacity and mount paths on supported hosts; Linux physical inventory follows. Never invent values; unsupported or failed probes visible. Mount, unmount, format and partition operations deferred until privileged service, authorization, explicit device identity and recovery validation.
- [ ] AERA-031 Settings groups appearance and system/storage actions; persistence later.
- [ ] AERA-032 First-party application registry supports launch and independent window state.
- [ ] AERA-033 File/storage services separate from widgets; no privileged actions in hosted mode.
- [ ] AERA-034 Guest boots to desktop with Files, Disks and Settings; evidence requires actual VM smoke plus manual interaction.
 
First increment: Rust standard library for file navigation; sysinfo 0.33.1 disk inventory for portable read-only mount capacity, no shell commands or elevation. Pin direct dependency, audit resolved graph before release. Synchronous bounded listing and disk refresh are preview limitations; move to background workers before production. Tests use temporary directories, verify navigation and missing paths, no user file writes. No separate human approval obtained in autonomous run.

## File creation and copy increment
- [ ] AERA-035 Create a folder or empty file in the displayed directory using a single validated name; existing targets are never overwritten. Reject separators, traversal, empty names, Windows reserved names and invalid characters. Show errors and refresh on success.
- [ ] AERA-036 Copy a selected regular file into the displayed directory with an explicit destination name and exclusive creation; reject directories, symlinks and special files; report partial destination on I/O failure and leave it for explicit recovery rather than risk deleting a replaced path. No automatic overwrite, rename, move or delete. Source is opened read-only; concurrent external path changes are not fully hardened in this prototype. Hosted writes explicitly requested through GUI only, under user permissions. No new dependency. Tests cover collision, invalid names, source preservation and copy contents.

## macOS-inspired desktop behavior
- [ ] AERA-037 Persistent top menu bar on normal desktops, with AERA menu, application shortcuts, workspace switcher and session indicator. Hide the bar and dock while an internal app is maximized; restore them on exit from maximized mode. Keep an obvious restore control and Escape shortcut.
- [ ] AERA-038 Each internal window has close, minimize, maximize and restore actions. Minimized windows remain in the dock and reopen without discarding application state. Maximize fills the AERA viewport; restore returns to the previous floating geometry. Only one maximized window per workspace. This does not maximize host windows.
- [ ] AERA-039 Multiple session workspaces, add and switch between up to eight; each retains its independent window visibility and maximize state. AERA menu and workspace switcher available in normal mode; keyboard Ctrl+Alt+Left/Right switches workspaces, including maximized mode. One instance of each app per workspace; app content initially shared across workspaces, separate application content and persistent workspace restoration are subsequent acceptance gates.
- [ ] AERA-040 Consistent macOS-inspired interaction grammar: top menu, bottom dock, compact title controls, restrained window chrome, light/dark themes, no copied Apple assets. Native compositor integration and external application management remain separate gates.

## Window and terminal correction
User runtime screenshot shows opaque windows and a height resize defect. Set independent resize axes and explicitly allocate available vertical content, with scroll for smaller sizes. Translucent themed frame is real alpha blending, not optical backdrop blur. Terminal is an ordinary first-party app with monospace output, explicit command submission and user privileges only; hosted mode runs on the host, not a sandbox. No auto-run. First version is a command runner, not yet a PTY emulator.
