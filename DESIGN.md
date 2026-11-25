# AERA visual system

AERA uses quiet, architectural graphite and porcelain surfaces, a restrained teal accent, generous spacing and a geometric wordmark. No imitation of macOS or Windows. Dark palette: canvas #101419, surfaces #1B222A, elevated #27313B, text #E8EDF2, accent #73D7C4. Light palette: canvas #F0F3F5, surfaces #FFFFFF, elevated #DDE5E9, text #202B32, accent #136C60. Use consistent 8/16/24 spacing and 8px corner radii. Status bar, dock, search and app windows must remain usable at 640x480 and high DPI. Wallpaper is procedural geometry, not a bitmap dependency. Respect theme contrast and reduced motion; no constant animation. AERA Shell is the same Rust application in hosted and guest sessions. Visual acceptance requires screenshots and interaction tests on actual display; no mockup substitutes.

## Desktop interaction grammar
Use familiar macOS-inspired top menu and dock layout, with compact window controls for close/minimize/maximize. Keep AERA's own colors, typography and identity. Top bar and dock disappear in maximized presentation; restore remains visible. Workspace buttons indicate active desktop and minimized apps remain identifiable in the dock. No copied Apple assets.

Liquid Glass is a selectively applied GPU material, not a global opacity effect. Keep application text sharp, provide opaque fallback and respect reduced motion. See GLASS.md for the rendering and screenshot contract.

## Visual correction — desktop first
User runtime screenshot confirms the centered shader demo and rectangular launcher tiles do not meet the reference. Remove the demo and duplicate launcher from the desktop; keep the optical test only in Settings. Normal desktop: full-area wallpaper, compact permanent top menu, centered floating rounded dock with vector app icons and running indicators, no marketing headline. Window controls are colored circles on the left, no duplicate title bar. Files gains real location sidebar. Glass-like chrome initially uses translucent fills and edge strokes; actual backdrop refraction remains a separate GPU integration gate. Verify screenshots from executable before claiming visual match.

## Fresh visual identity (supersedes teal palette)
Default light: porcelain, cool blue and lavender wallpaper with warm peach highlights; dark: ink, slate and muted violet. Accent blue, not teal. App icons use a common rounded silhouette with distinct restrained blue, silver, violet and amber accents. Native vector glyphs on a 24-unit grid; 16/20 logical px for menus and settings, 32/40 for dock, adjustable compact dock. Keep strokes proportional and labels readable. Twelve named symbols: Files, Disks, Settings, About, Terminal, Monitor, App Library, Installer, Search, Notifications, Network, Appearance. Only implemented apps launch; reserved symbols do not imply functionality. No generated icon board as runtime asset or screenshot.

## Wallpapers
Desktop backgrounds are user-selectable independently of appearance. Ship original procedural landscape artwork as the default and alternate abstract and dusk choices. Custom local PNG/JPEG/WebP images supported with centered cover crop or fit. Decode bounded images, upload once per selection, preserve previous wallpaper on failure. No network fetch, no fake screenshot. First increment session-only settings; persistence and per-workspace choices later.
