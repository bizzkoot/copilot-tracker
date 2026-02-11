# Changelog

## [2.3.3](https://github.com/bizzkoot/copilot-tracker/compare/v2.3.2...v2.3.3) (2026-02-11)


### Bug Fixes

* add robust polling restart with debounce and shutdown protection ([1cc6061](https://github.com/bizzkoot/copilot-tracker/commit/1cc60611e4050ef170ef7a82a84d7b099fed45dd))
* update widget to use Tauri 2.x official API ([0223424](https://github.com/bizzkoot/copilot-tracker/commit/0223424156c63ffbce2cb065c99c1fcfc0571c3d))
* **widget:** ensure widget fills full window height to prevent white bottom bar ([8e453ac](https://github.com/bizzkoot/copilot-tracker/commit/8e453ac21bcea71997dd2caa4ece3f22f69e04e9))
* **widget:** fetch cached usage data on mount to prevent race condition ([54f6f89](https://github.com/bizzkoot/copilot-tracker/commit/54f6f896d139c4cc5b2cdc667784e2f68a912530))
* **widget:** fix position initialization and restore usage data display ([af8e6c0](https://github.com/bizzkoot/copilot-tracker/commit/af8e6c054662b7cd3a73608f3b9b912eedb0ee3c))
* **widget:** improve dragging, focus handling, and tray menu sync ([20cc42f](https://github.com/bizzkoot/copilot-tracker/commit/20cc42fa287a923dd9380d0c0de5d63dc4f0ca2f))
* **widget:** persist position and sync state between tray and settings ([ec42148](https://github.com/bizzkoot/copilot-tracker/commit/ec42148331b2c1c2506826abfdaca97cf00e107d))


### Documentation

* **widget:** add new widget image asset ([01d8385](https://github.com/bizzkoot/copilot-tracker/commit/01d83852bb8ca33f2d0b3d88c43f2b0bb1148309))

## [2.3.2](https://github.com/bizzkoot/copilot-tracker/compare/v2.3.1...v2.3.2) (2026-02-10)


### Bug Fixes

* **build:** include widget entry in renderer build - release ([be93fce](https://github.com/bizzkoot/copilot-tracker/commit/be93fce9c24befb22b3f431313e64568b4de5ce5))
* **widget:** update positioning API and improve notification handling ([d37444f](https://github.com/bizzkoot/copilot-tracker/commit/d37444f3a5aea4038b2bbe70b5ac7e98fd27b20c))

## [2.3.1](https://github.com/bizzkoot/copilot-tracker/compare/v2.3.0...v2.3.1) (2026-02-10)


### Bug Fixes

* **ci:** use PAT for release-please to trigger PR checks automatically ([89fae57](https://github.com/bizzkoot/copilot-tracker/commit/89fae579c8fc5e37afbe86f472e58013af127a10))
* **code-quality:** resolve audit findings from P0 to P3-Low ([10a933f](https://github.com/bizzkoot/copilot-tracker/commit/10a933fd19e6b1cc6f4d70c683e0514d338c63f6))
* resolve build errors and add comprehensive PR checks ([de023db](https://github.com/bizzkoot/copilot-tracker/commit/de023db9799cee7d4197c0c1042099427506889b))


### Other

* trigger release-please PR creation ([1ce840f](https://github.com/bizzkoot/copilot-tracker/commit/1ce840f301408b4f946258259bda2b48f91d1490))

## [2.3.0](https://github.com/bizzkoot/copilot-tracker/compare/v2.2.0...v2.3.0) (2026-02-10)


### Features

* **widget:** add floating usage widget with state persistence ([6a7ff9c](https://github.com/bizzkoot/copilot-tracker/commit/6a7ff9cb4323bfd5835e613ab6cb1590cc9ebf44))


### Bug Fixes

* **core:** improve startup and init robustness ([70480d4](https://github.com/bizzkoot/copilot-tracker/commit/70480d41b7956d6080dae5c545430bd846f7a330))
* **tray:** ensure Open Dashboard navigates to main page ([c78ac8c](https://github.com/bizzkoot/copilot-tracker/commit/c78ac8c6d3a362911e1da36bffc417b431a86fb5))

## [2.2.0](https://github.com/bizzkoot/copilot-tracker/compare/v2.1.2...v2.2.0) (2026-02-08)

### Features

- Add model-level usage breakdown with expandable history table ([6f4760b](https://github.com/bizzkoot/copilot-tracker/commit/6f4760b39edeb07a1f3440d1dad1eefc8e3bb789))
- **dashboard:** add model usage totals to Daily Breakdown table footer ([c4ef228](https://github.com/bizzkoot/copilot-tracker/commit/c4ef228ed669c9e42ad536b768c2f60e246d7a2d))
- **ui/backend:** enhance dashboard, unify prediction logic, and fix theme sync ([e5737c6](https://github.com/bizzkoot/copilot-tracker/commit/e5737c6a9af853087f0dff6b2c2953124cb0e14f))
- **ui/dashboard:** release - enhance dashboard with EMA trend line and compact layout ([94e7d67](https://github.com/bizzkoot/copilot-tracker/commit/94e7d67016db08dc5b071e38323cb33d848749ff))

### Bug Fixes

- **dev:** prevent port 5173 conflict in tauri dev mode ([04dc9cb](https://github.com/bizzkoot/copilot-tracker/commit/04dc9cbd8fda4f5e65b76c7336984631a2427028))
- resolve date parsing issue and refactor usage fetching ([f552c74](https://github.com/bizzkoot/copilot-tracker/commit/f552c743bfb5d7a9f4fbd2aca3d5ee65ed61f4c8))
- **settings:** prevent theme race condition on window focus/refresh ([a2f9e07](https://github.com/bizzkoot/copilot-tracker/commit/a2f9e079cb91abf9163b9b5ac524b6aa8f599e30))

## [2.1.2](https://github.com/bizzkoot/copilot-tracker/compare/v2.1.1...v2.1.2) (2026-02-07)

### Bug Fixes

- **ci:** correct version extraction regex in release workflow ([3b464de](https://github.com/bizzkoot/copilot-tracker/commit/3b464de3414e5d557cd671cafee52efb2baa8da9))
- **ci:** release - correct version verification regex and non-blocking Cargo.lock check ([1f1287e](https://github.com/bizzkoot/copilot-tracker/commit/1f1287e718ed9970cc734142b83dc78f7d633ae8))

## [2.1.1](https://github.com/bizzkoot/copilot-tracker/compare/v2.1.0...v2.1.1) (2026-02-07)

### Bug Fixes

- add x-release-please-version marker to Cargo.toml ([b8ed7dd](https://github.com/bizzkoot/copilot-tracker/commit/b8ed7dd4dbcb2404f627a09d0a3a7c78f4ad00b2))
- **ci:** add bootstrap-sha to release-please config ([5ad5285](https://github.com/bizzkoot/copilot-tracker/commit/5ad5285039d23453d7c30606407d3c62e53bc078))
- **ci:** correct changelog-sections configuration ([737172f](https://github.com/bizzkoot/copilot-tracker/commit/737172f592d2b754217871a1f3d5154a500d8523))
- **ci:** correct release-please tag naming convention ([d5358ae](https://github.com/bizzkoot/copilot-tracker/commit/d5358ae0efd07d47df87c4629de56b868385df33))
- **ci:** switch to release-please Manifest Mode ([c733f25](https://github.com/bizzkoot/copilot-tracker/commit/c733f25923df49a767684ec82bc821ef3a541fc4))
- **ci:** update bootstrap-sha to skip breaking change ([d853635](https://github.com/bizzkoot/copilot-tracker/commit/d853635efb6781adca4fd482c0e00b4839eb0584))
- **ci:** update version sync configuration for future builds ([69940ad](https://github.com/bizzkoot/copilot-tracker/commit/69940ad8d02e75ecbfbae3ce6b9ffbe674fd718d))
- correct Cargo.toml updater in release-please config ([55de77f](https://github.com/bizzkoot/copilot-tracker/commit/55de77f7024c650be4c2758bca9fcf7473dd03f7))
- **docs:** correct capitalization in README features list ([f980221](https://github.com/bizzkoot/copilot-tracker/commit/f9802214b5848ced2586c2134f21100d0d18561b))
- switch release-please config to Single Package Mode ([3e9d766](https://github.com/bizzkoot/copilot-tracker/commit/3e9d76642977fe777b49037b586d91c0f507f03b))
- **ui:** improve error message when usage data is unavailable ([72aefd2](https://github.com/bizzkoot/copilot-tracker/commit/72aefd2f8aea44bd4d8a482e6b2b38ced863e26f))

## [2.1.0](https://github.com/bizzkoot/copilot-tracker/compare/v2.0.1...v2.1.0) (2026-02-07)

### Features

- **polling:** add dynamic background polling with lifecycle management ([f07bc53](https://github.com/bizzkoot/copilot-tracker/commit/f07bc53247f7665208a857ae5f8cd7aefd05a9e4))

### Bug Fixes

- **platform:** resolve Linux window management and improve cross-platform compatibility ([f4102f3](https://github.com/bizzkoot/copilot-tracker/commit/f4102f3adf3f4da121fc284cd8f647f7010c0a30))
- **runtime:** use tauri::async_runtime::spawn instead of tokio::spawn ([45985a3](https://github.com/bizzkoot/copilot-tracker/commit/45985a32da7f820bf652fec0b35e71b9520fb9a8))

## [2.0.1](https://github.com/bizzkoot/copilot-tracker/compare/v2.0.0...v2.0.1) (2026-02-07)

### Bug Fixes

- sync Tauri version to 2.0.0 and add automated version sync workflow ([a6d0a19](https://github.com/bizzkoot/copilot-tracker/commit/a6d0a19950fa8268423528ca1e3c73bc40bbe46d))

## [2.0.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.5.1...v2.0.0) (2026-02-07)

### ⚠ BREAKING CHANGES

- Electron builds discontinued after v1.5.1

### Features

- add customizable tray icon display formats ([6972b39](https://github.com/bizzkoot/copilot-tracker/commit/6972b39a8342c72138337f87849da9327b6228d4))
- **tray:** enhance system tray menu with consumption metrics ([6ca57f4](https://github.com/bizzkoot/copilot-tracker/commit/6ca57f4801625ee7c798fab07b64ba0df014a2ad))

### Bug Fixes

- explicitly specify TOML type and jsonpath for Cargo.toml ([33fe842](https://github.com/bizzkoot/copilot-tracker/commit/33fe842fe17c75eb132a35e5ea8d2c2e803389c5))
- sync version to 1.5.1 and improve cross-platform compatibility ([e9abcfd](https://github.com/bizzkoot/copilot-tracker/commit/e9abcfd508d1ff9eab11345ae98a236af378c5ae))

### Continuous Integration

- remove Electron builds and update release workflow ([c5aacbf](https://github.com/bizzkoot/copilot-tracker/commit/c5aacbf198a7f243add9b2161f53fe2fad00959e))

## [1.5.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.5.0...v1.5.1) (2026-02-05)

### Bug Fixes

- properly hide dock icon on macOS using set_activation_policy ([2d957d5](https://github.com/bizzkoot/copilot-tracker/commit/2d957d57b90a8786bc1ce1a24ca43968803efe10))
- **release:** correct release-please configuration for Tauri version bumping ([6ab6731](https://github.com/bizzkoot/copilot-tracker/commit/6ab6731599b2aa0c5b5542d578cd223a995cb8a8))
- resolve tray-dashboard sync, dock visibility, and startup data issues ([75262c0](https://github.com/bizzkoot/copilot-tracker/commit/75262c0af74bb5fe3cc34cd0def6b36e8dad586a))

## [1.5.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.4.2...v1.5.0) (2026-02-05)

### Features

- **tray:** implement Retina-aware text rendering with improved sharpness ([7a2b30c](https://github.com/bizzkoot/copilot-tracker/commit/7a2b30c015cd66b9ec0b85b9a40e61f15197bfe5))

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
