# Nephtys
Selfhosted Home Safety Camera software built with Rust, Svelte, FFMPEG & OpenCV
Nephtys is designed for local deployement with a .local domain using zeroconf mDNS
Nephtys can be deployed to capture almost any supported linux webcams.

## Goals
- [x] HLS Streaming one camera
- [ ] ~~Save last 4 days to disk~~ (cancelled)
- [ ] Able to record from one or more sources
- [x] Movement and person detection
- [x] Automatic save when movement detection
- [x] Remote control (Web UI)
    - Basic stream view (done)
    - Event logs (movement detection)
    - View past recordings
- [ ] Home Assistant integration
- [ ] Automatic push notifications alerts (via HA)
