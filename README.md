<div align="center">

# File transfer doesn't need to be complicated

</div>



<div align="center">

![AlterSendme working demo](assets/animation.gif)

</div>

<div align="center">
    <a href="https://github.com/bruceblink/alter-sendme/releases/latest" target="_blank">
        <img src="https://img.shields.io/github/v/release/bruceblink/alter-sendme" alt="release-version">
    </a>
    <!-- total download -->
    <a href="https://github.com/bruceblink/alter-sendme/releases/latest" target="_blank">
        <img src="https://img.shields.io/github/downloads/bruceblink/alter-sendme/total" alt="downloads">
    </a>
    <!-- stars -->
    <a href="https://github.com/bruceblink/alter-sendme/stargazers" target="_blank">
        <img src="https://img.shields.io/github/stars/bruceblink/alter-sendme?" alt="stars">
    </a>
    <!-- forks -->
    <a href="https://github.com/bruceblink/alter-sendme/network/members" target="_blank">
        <img src="https://img.shields.io/github/forks/bruceblink/alter-sendme.svg?" alt="fork">
    </a>
    <!-- visitors -->
    <a href="https://github.com/bruceblink/alter-sendme" target="_blank">
        <img src="https://api.visitorbadge.io/api/visitors?path=https%3A%2F%2Fgithub.com%2Fbruceblink%2Fani-todo-app&countColor=%23263759&style=flat"
            alt="visitors">
    </a>
    <!-- license -->
    <a href="https://github.com/bruceblink/alter-sendme/blob/main/LICENSE" target="_blank">
        <img src="https://img.shields.io/github/license/bruceblink/alter-sendme" alt="license">
    </a>
    <!-- sponsor -->
    <a href="https://github.com/bruceblink/alter-sendme/blob/main/LICENSE" target="_blank">
        <img src="https://img.shields.io/badge/sponsor-ff69b4" alt="sponsor">
    </a>
    <br>
    <!-- window -->
    <a href="https://github.com/bruceblink/alter-sendme/releases/latest" target="_blank">
        <img src="https://img.shields.io/badge/Windows-0078D6?style=flat&logo=windows&logoColor=white" alt="window">
    </a>
    <!-- macos -->
    <a href="https://github.com/bruceblink/alter-sendme/releases/latest" target="_blank">
        <img src="https://img.shields.io/badge/MACOS-adb8c5?style=flat&logo=macos&logoColor=white" alt="macos">
    </a>
    <!-- linux -->
    <a href="https://github.com/bruceblink/alter-sendme/releases/latest" target="_blank">
        <img src="https://img.shields.io/badge/linux-1793D1?style=flat&logo=linux&logoColor=white" alt="linux">
    </a>
    <br>
    <a href="https://www.buymeacoffee.com/bruceblink" target="_blank"><img
            src="https://cdn.buymeacoffee.com/buttons/v2/default-blue.png" alt="Buy Me A Coffee"
            style="height: 40px !important;width: 145px !important;">
    </a>

</div>

This project is based on [alt-sendme](https://github.com/tonyantony300/alt-sendme). 

It's a free and open-source file transfer tool that harnesses the power of [cutting-edge peer-to-peer networking](https://www.iroh.computer), letting you transfer files directly without storing them on cloud servers.

Why rely on WeTransfer, Dropbox, or Google Drive when you can reliably and easily transfer files directly, end-to-end encrypted and without revealing any personal information?


## Features

- **Send anywhere** – Works seamlessly on local networks or across continents.
- **Peer-to-peer direct transfer** – Send files straight between devices, with no cloud storage in between.
- **End-to-end encryption** – Always-on protection with QUIC + TLS 1.3 for forward and backward secrecy.
- **No accounts or personal info** – Transfer files without sign-ups or exposing private data.
- [**Transfer anything**](https://www.iroh.computer/proto/iroh-blobs) – Send files or directories of any size or any format, verified with BLAKE3-based integrity checks.
- **Resumable transfers** – Interrupted downloads automatically resume where they left off.
- **Fast & reliable** – Capable of saturating multi-gigabit connections for lightning-fast transfers.
- [**NAT traversal via QUIC**](https://www.iroh.computer/docs/faq#does-iroh-use-relay-servers) – Secure, low-latency connections using QUIC hole punching with encrypted relay fallback.
- **Free & open source** – No upload costs, no size limits, and fully community-driven.



## Installation

The easiest way to get started is by downloading one of the following versions for your respective operating system:

<table>
  <tr>
    <td><b>Platform</b></td>
    <td><b>Download</b></td>
  </tr>
  <tr>
    <td><b>Windows</b></td>
    <td><a href='https://github.com/bruceblink/alter-sendme/releases/download/v0.1.5/AlterSendme_0.1.5_x64-setup.exe'>AlterSendme.exe</a></td>
  </tr>
  <tr>
    <td><b>macOS</b></td>
    <td><a href='https://github.com/bruceblink/alter-sendme/releases/download/v0.1.5/AlterSendme_0.1.5_universal.dmg'>AlterSendme.dmg</a></td>
  <tr>
    <td><b>Linux </b></td>
    <td><a href='https://github.com/bruceblink/alter-sendme/releases/download/v0.1.5/AlterSendme_0.1.5_amd64.deb'>AlterSendme.deb</a></td>
  </tr>
</table>


More download options in [GitHub Releases](https://github.com/bruceblink/alter-sendme/releases).


## Supported Languages


- 🇫🇷 French
- 🇹🇭 Thai
- 🇩🇪 German
- 🇨🇳 Chinese
- 🇯🇵 Japanese
- 🇷🇺 Russian
- 🇨🇿 Czech
- 🇮🇹 Italian
- 🇸🇦 Arabic
- 🇧🇷 Portuguese (Brazilian)
- 🇰🇷 Korean
- 🇪🇸 Spanish



## Development

If you want to contribute or run the app from source:

### Prerequisites

- Rust 1.85+
- Node.js 22+
- pnpm

### Running in Development

1. **Install frontend dependencies**:
   ```bash
   npm install -g pnpm
   pnpm install
   ```

2. **Run the desktop app**:
   ```bash
   pnpm tauri dev # or cargo tauri dev
   ```

This will start the app with hot reload enabled for both frontend and backend changes.


### Building Locally


 1. **Build stage**:
   ```bash
     pnpm tauri build # or cargo tauri build
   ```
 2. **Run**:

  ```bash
    cd ./target/release
    ./alter-sendme        # macOS or Linux
    alter-sendme.exe      # Windows
   ```


## License

[AGPL-3.0](LICENSE)

## Privacy Policy

See [PRIVACY.md](PRIVACY.md) for information about how AlterSendme handles your data and privacy.

[![Sponsor](https://img.shields.io/badge/sponsor-30363D?style=for-the-badge&logo=GitHub-Sponsors&logoColor=#EA4AAA)](https://github.com/sponsors/bruceblink) [![Buy Me Coffee](https://img.shields.io/badge/Buy%20Me%20Coffee-FF5A5F?style=for-the-badge&logo=coffee&logoColor=FFFFFF)](https://buymeacoffee.com/bruceblink)


## Contributors

<a href="https://github.com/bruceblink/alter-sendme/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=bruceblink/alter-sendme"  alt="bruceblink/alter-sendme"/>
</a>


## Acknowledgements

- [Tauri](https://v2.tauri.app)
- [Sendmer](https://crates.io/crates/sendmer)

## Contact 


Thank you for checking out this project! If you find it useful, consider giving it a star and helping spread the word.


[badge-version]: https://img.shields.io/badge/version-0.1.5-blue
[badge-platforms]: https://img.shields.io/badge/platforms-macOS%2C%20Windows%2C%20Linux%2C%20-green
[badge-sponsor]: https://img.shields.io/badge/sponsor-ff69b4
[badge-hire]: https://img.shields.io/badge/hire%20developer-8b5cf6


