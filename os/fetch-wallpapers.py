"""Import the three reviewed CC0 photos; no runtime download."""
from pathlib import Path
import urllib.request
import hashlib
import os

ROOT = Path(__file__).resolve().parents[1] / "crates/aera-shell/assets/wallpapers"
PHOTOS = {
    "alpine.jpg": "https://upload.wikimedia.org/wikipedia/commons/b/bc/Mountains_by_the_lake_%28Unsplash%29.jpg",
    "midnight.jpg": "https://upload.wikimedia.org/wikipedia/commons/9/9b/Starry_sky_over_a_mountain_and_a_lake_%28Unsplash%29.jpg",
    "autumn.jpg": "https://upload.wikimedia.org/wikipedia/commons/5/5b/Mountain_lake_%28Unsplash%29.jpg",
}
ROOT.mkdir(parents=True, exist_ok=True)
for name, url in PHOTOS.items():
    path = ROOT / name
    req = urllib.request.Request(url, headers={"User-Agent": "AERA-WallpaperImporter/1.0 (https://github.com/GaryWenneker/AERA)"})
    with urllib.request.urlopen(req, timeout=60) as response:
        data = response.read(12 * 1024 * 1024 + 1)
    if len(data) > 12 * 1024 * 1024 or not data.startswith(b"\xff\xd8\xff") or not data.endswith(b"\xff\xd9"):
        raise ValueError(f"Invalid JPEG: {name}")
    if name == "midnight.jpg" and hashlib.sha1(data).hexdigest() != "628108fa09a77c75cb715b76325abd739542a5d9":
        raise ValueError("Unexpected source revision: midnight.jpg")
    temp = path.with_suffix(".tmp")
    temp.write_bytes(data)
    os.replace(temp, path)
    print(name, len(data), hashlib.sha256(data).hexdigest())
