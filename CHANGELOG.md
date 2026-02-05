# Changelog

## [1.5.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.5.0...v1.5.1) (2026-02-05)


### Bug Fixes

* properly hide dock icon on macOS using set_activation_policy ([2d957d5](https://github.com/bizzkoot/copilot-tracker/commit/2d957d57b90a8786bc1ce1a24ca43968803efe10))
* **release:** correct release-please configuration for Tauri version bumping ([6ab6731](https://github.com/bizzkoot/copilot-tracker/commit/6ab6731599b2aa0c5b5542d578cd223a995cb8a8))
* resolve tray-dashboard sync, dock visibility, and startup data issues ([75262c0](https://github.com/bizzkoot/copilot-tracker/commit/75262c0af74bb5fe3cc34cd0def6b36e8dad586a))

## [1.5.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.4.2...v1.5.0) (2026-02-05)


### Features

* **tray:** implement Retina-aware text rendering with improved sharpness ([7a2b30c](https://github.com/bizzkoot/copilot-tracker/commit/7a2b30c015cd66b9ec0b85b9a40e61f15197bfe5))

## [1.4.2](https://github.com/bizzkoot/copilot-tracker/compare/v1.4.1...v1.4.2) (2026-02-04)

### Bug Fixes

- **electron:** prevent auth window flash and reload loop on Windows ([891e29a](https://github.com/bizzkoot/copilot-tracker/commit/891e29acef7304d45fc3740c263401c42ed047d0))
- **tauri:** resolve Windows and Linux build failures ([d4fd9b2](https://github.com/bizzkoot/copilot-tracker/commit/d4fd9b22afa604433137470a9ddac979e953cf00))

## [1.4.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.4.0...v1.4.1) (2026-02-04)

### Bug Fixes

- add bundle metadata for Windows Tauri build ([611d318](https://github.com/bizzkoot/copilot-tracker/commit/611d318c74e2a137d3ed4daa01f89db0b5a3f71d))
- **electron:** resolve Windows auth window and clean code - release ([fe102ab](https://github.com/bizzkoot/copilot-tracker/commit/fe102abef3939ffc179c0a259d3c53628d8dbe18))
- remove icon array from tauri.conf.json to allow auto-detection of platform icons ([9f80fe6](https://github.com/bizzkoot/copilot-tracker/commit/9f80fe6db080ea60cefa811998773c8950ea22a0))

## [1.4.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.3.1...v1.4.0) (2026-02-04)

### Features

- add multiplatform Tauri builds to CI/CD workflow ([069f461](https://github.com/bizzkoot/copilot-tracker/commit/069f461c528e659fc0a46a0f21e89cb0438c9f8c))
- **auth:** implement auth-time usage data capture via URL hash redirect ([ec981c5](https://github.com/bizzkoot/copilot-tracker/commit/ec981c5432595c62a21c2c9f75ca3fe5f9a55685))
- **auth:** implement working hidden webview for silent usage refresh ([0299bf0](https://github.com/bizzkoot/copilot-tracker/commit/0299bf0a9ab524a378465804c5540869c1a82a45))
- **docs:** add PRD for Tauri migration of Copilot Tracker ([c153360](https://github.com/bizzkoot/copilot-tracker/commit/c1533603ca5d8857e83b945f52856660c196e025))
- **refresh:** update tray refresh to trigger re-authentication ([1260fc9](https://github.com/bizzkoot/copilot-tracker/commit/1260fc957987fb620f30964f627cdeabc1ec1a85))
- **tauri:** add linting step to tauri build process ([552a9bb](https://github.com/bizzkoot/copilot-tracker/commit/552a9bb1f1f49b67a353efec1af39a10969286e4))
- **tauri:** complete feature parity with Electron implementation ([c7cbe61](https://github.com/bizzkoot/copilot-tracker/commit/c7cbe610ee15aa6e88ed982a3bfcc6784472b5c2))
- **tauri:** complete migration implementation and build support ([78b892c](https://github.com/bizzkoot/copilot-tracker/commit/78b892c95323260059d6c7dc3153d28713ee004b))
- **tauri:** migrate authentication and dashboard logic from Electron ([f24e5ac](https://github.com/bizzkoot/copilot-tracker/commit/f24e5ac85d33daabb8426af881c3163de0289400))
- **tray:** implement Electron-style tray icon with progress indicator ([07141ac](https://github.com/bizzkoot/copilot-tracker/commit/07141ac27741d5972e70f62925e6371e2d56bd67))

### Bug Fixes

- **auth:** implement custom protocol redirect for safer data extraction ([f3024ad](https://github.com/bizzkoot/copilot-tracker/commit/f3024ad7aafcee89e576951c9db22680ea59be24))
- **auth:** implement single-window extraction with custom protocol redirect and auto-navigation ([3a47db9](https://github.com/bizzkoot/copilot-tracker/commit/3a47db9590721b80a3eb6d19381b8065f97f26ec))
- **data:** implement camelCase JSON parsing and persistent history storage ([b542216](https://github.com/bizzkoot/copilot-tracker/commit/b5422162cb7b87e6b5df501f862d9cc66a0843bc))
- **reset:** fix race condition and ensure auth state properly updates ([ecc93f0](https://github.com/bizzkoot/copilot-tracker/commit/ecc93f00f2072b778119acb1106e419eb87d8ff5))
- **reset:** properly clear all data on Reset and logout frontend ([68fa4a7](https://github.com/bizzkoot/copilot-tracker/commit/68fa4a785bc2da850fad8816797cd14c1a3f4d44))
- **reset:** wire Reset button to actual backend reset function ([886e558](https://github.com/bizzkoot/copilot-tracker/commit/886e558683e841c8ef5dbf7daf21034c92e0609e))
- **tauri:** authentication flow improvements and clippy cleanup ([bf5387a](https://github.com/bizzkoot/copilot-tracker/commit/bf5387a244f8af8c52155bf4b56399f37177a2f0))
- **tauri:** resolve 504MB binary size issue ([ad9ff57](https://github.com/bizzkoot/copilot-tracker/commit/ad9ff573a27e882949b376193fff51c268f37c9b))
- **tray:** CRITICAL BUG FIX - tray listener was parsing wrong event type ([4989ed9](https://github.com/bizzkoot/copilot-tracker/commit/4989ed9512adffd6705c9290bcd9bf8341bb9e7e))
- **tray:** remove progress circle from tray icon ([c43bf01](https://github.com/bizzkoot/copilot-tracker/commit/c43bf0115562401937d5e9a740ef88b7c62df9a4))
- **tray:** synchronize tray icon with dashboard in real-time ([a919fca](https://github.com/bizzkoot/copilot-tracker/commit/a919fca1ad3423e1f1fe0b09244d44ed79af3a00))
- **ui:** resolve all 5 dashboard and tray synchronization issues ([899f7bf](https://github.com/bizzkoot/copilot-tracker/commit/899f7bff2d5be13aefc591652acef83d8a4d558b))

## [1.3.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.3.0...v1.3.1) (2026-02-03)

### Bug Fixes

- **auth:** prevent invisible Re-Login window on Windows and stop unauthenticated GitHub polling - release ([b744aac](https://github.com/bizzkoot/copilot-tracker/commit/b744aacae9e276727195f2f75e1b13aab4450118))

## [1.3.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.2.0...v1.3.0) (2026-02-01)

### Features

- **tray:** add monthly prediction banner and dashboard action - release ([a8b4081](https://github.com/bizzkoot/copilot-tracker/commit/a8b4081db3f96d78250b0d1aa593387bfb996360))

### Bug Fixes

- startup authentication and auto-minimize on launch ([6cc332e](https://github.com/bizzkoot/copilot-tracker/commit/6cc332efe68f8ace86a5e60b16253a536bbac03a))

## [1.2.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.1.2...v1.2.0) (2026-02-01)

### Features

- add update notification system with native alerts and tray indicators ([edc2666](https://github.com/bizzkoot/copilot-tracker/commit/edc2666f2d73095188e15db9b52e45029652dce7))

## [1.1.2](https://github.com/bizzkoot/copilot-tracker/compare/v1.1.1...v1.1.2) (2026-02-01)

### Bug Fixes

- sync settings UI with tray and improve canvas types ([1eff249](https://github.com/bizzkoot/copilot-tracker/commit/1eff24942d7bd3ef31459e3c501b79f7339c2510))

## [1.1.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.1.0...v1.1.1) (2026-02-01)

### Bug Fixes

- upload release assets using tag_name output instead of github.ref ([4f68eb6](https://github.com/bizzkoot/copilot-tracker/commit/4f68eb6233bedd6dbc2c65ec84408b5a8cff2b65))

## [1.1.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.0.0...v1.1.0) (2026-02-01)

### Features

- add asset validation script ([1afe178](https://github.com/bizzkoot/copilot-tracker/commit/1afe1785775ca3ea3a137531c2662535064d12e7))
- add environment variables configuration and dev-only logging ([f1b99c9](https://github.com/bizzkoot/copilot-tracker/commit/f1b99c9fbb42c93df6816cf4e225eb3c732ed858))

### Bug Fixes

- add macOS traffic light safe area and update dashboard icon ([22f097a](https://github.com/bizzkoot/copilot-tracker/commit/22f097acc9893d93c2f79634bb5223020fe82b70))
- configure app icons and resolve production asset loading ([fe84671](https://github.com/bizzkoot/copilot-tracker/commit/fe84671943307c7bfa0e244928bf782505b2d1a5))
- configure canvas dependency with fallback ([66f78ba](https://github.com/bizzkoot/copilot-tracker/commit/66f78baa98c6376815eedba21126776fbc9cac26))
- modernize Electron API and fix app exit crashes ([36aeaf6](https://github.com/bizzkoot/copilot-tracker/commit/36aeaf610948fb0629fda0829addd7a9a5c6b977))
- remove global keyboard shortcuts and hide app from macOS dock ([ec1c0f7](https://github.com/bizzkoot/copilot-tracker/commit/ec1c0f7f69db662001f62b15956aa80521f5754d))
- resolve history parsing and renderer display issues ([067c1f1](https://github.com/bizzkoot/copilot-tracker/commit/067c1f1323b3edf3be7723db853d53b3e496a9aa))
- **tray:** navigation to settings and optimize icon generation ([514699e](https://github.com/bizzkoot/copilot-tracker/commit/514699e596dd84fa8ae19bdac44d9de38796e9f2))

## 1.0.0 (2026-02-01)

### Features

- add asset validation script ([1afe178](https://github.com/bizzkoot/copilot-tracker/commit/1afe1785775ca3ea3a137531c2662535064d12e7))
- add environment variables configuration and dev-only logging ([f1b99c9](https://github.com/bizzkoot/copilot-tracker/commit/f1b99c9fbb42c93df6816cf4e225eb3c732ed858))

### Bug Fixes

- add macOS traffic light safe area and update dashboard icon ([22f097a](https://github.com/bizzkoot/copilot-tracker/commit/22f097acc9893d93c2f79634bb5223020fe82b70))
- configure app icons and resolve production asset loading ([fe84671](https://github.com/bizzkoot/copilot-tracker/commit/fe84671943307c7bfa0e244928bf782505b2d1a5))
- configure canvas dependency with fallback ([66f78ba](https://github.com/bizzkoot/copilot-tracker/commit/66f78baa98c6376815eedba21126776fbc9cac26))
- modernize Electron API and fix app exit crashes ([36aeaf6](https://github.com/bizzkoot/copilot-tracker/commit/36aeaf610948fb0629fda0829addd7a9a5c6b977))
- remove global keyboard shortcuts and hide app from macOS dock ([ec1c0f7](https://github.com/bizzkoot/copilot-tracker/commit/ec1c0f7f69db662001f62b15956aa80521f5754d))
- resolve history parsing and renderer display issues ([067c1f1](https://github.com/bizzkoot/copilot-tracker/commit/067c1f1323b3edf3be7723db853d53b3e496a9aa))
- **tray:** navigation to settings and optimize icon generation ([514699e](https://github.com/bizzkoot/copilot-tracker/commit/514699e596dd84fa8ae19bdac44d9de38796e9f2))
