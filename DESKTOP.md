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
