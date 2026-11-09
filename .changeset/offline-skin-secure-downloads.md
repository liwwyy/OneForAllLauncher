---
default: patch
---

Keep local offline skins working when Secure Skins upgrades their loopback URL to HTTPS. Restore only the exact private PNG endpoint at URL creation, preserve remote TLS behavior, and add a regression using the real mod's rewrite method.
